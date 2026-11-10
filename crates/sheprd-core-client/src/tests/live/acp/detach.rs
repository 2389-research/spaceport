// ABOUTME: Verifies that Core owns the adapter while all of its viewers detach.
// ABOUTME: Reconnects to the same live process and continues its ACP conversation.

use super::*;
use crate::wire::TerminalState;
use sheprd_chat_protocol::{Cursor, Status, Transcript};

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY and SHEPRD_ACP_TEST_BINARY"]
fn a_core_owned_adapter_survives_all_clients_detaching() {
    let core_binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let adapter_binary =
        PathBuf::from(std::env::var_os("SHEPRD_ACP_TEST_BINARY").expect("SHEPRD_ACP_TEST_BINARY"));
    assert!(adapter_binary.is_absolute() && adapter_binary.is_file());
    let mut sandbox = Sandbox::start(core_binary);
    let private_dir = sandbox._dir.path().to_owned();
    let core = sandbox.connect();
    let endpoint = EndpointId::random();
    let peer = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/acp_peer.py");
    let argv = [
        adapter_binary.to_str().unwrap().to_owned(),
        "--headless".to_owned(),
        "--".to_owned(),
        "python3".to_owned(),
        "-I".to_owned(),
        peer.to_str().unwrap().to_owned(),
    ];
    let terminal = core
        .run(&Run {
            argv: &argv,
            env: [(sheprd_chat_protocol::ENDPOINT_ENV, endpoint.as_str())].into(),
            labels: [(sheprd_chat_protocol::TOKEN, endpoint.as_str())].into(),
            keep: false,
            name: Some("private-acp-detach"),
            ..Run::default()
        })
        .unwrap();
    let pid = terminal.pid.expect("Core identifies the adapter process");
    eprintln!(
        "private Core pid {}, adapter pid {pid}, terminal {}, endpoint {endpoint}, sandbox {}",
        sandbox.child.id(),
        terminal.address,
        private_dir.display()
    );
    assert!(!terminal.keep);
    let attachment = core
        .attach(&Attach {
            observe: true,
            terminal: &terminal.address,
            ..Attach::default()
        })
        .unwrap();
    assert_eq!(attachment.terminal.pid, Some(pid));
    let address = sandbox
        ._dir
        .path()
        .join("state/herdr/acp")
        .join(format!("{endpoint}.sock"));
    let deadline = Instant::now() + WAIT;
    while !address.exists() {
        assert!(Instant::now() < deadline, "adapter did not open its socket");
        assert_eq!(
            attachment.read(Duration::from_millis(100)).unwrap().end,
            None,
            "adapter exited before accepting chat clients"
        );
    }
    let mut first = ChatClient::attach(&address);
    let (before_cursor, mut transcript) = snapshot(&mut first);
    wait_for(&mut first, &mut transcript, |transcript| {
        transcript.status == Status::Idle && transcript.session.is_some()
    });
    let mut second = ChatClient::attach(&address);
    let (_, mut second_transcript) = snapshot(&mut second);
    wait_for(&mut second, &mut second_transcript, |transcript| {
        transcript.clients == 2
    });
    complete_turn(&mut first, &mut transcript, "before detach", "allow");
    let entries = transcript.entries.clone();
    let session = transcript.session.clone();
    assert!(entries.iter().any(
        |entry| matches!(&entry.kind, EntryKind::User { text, .. } if text == "before detach")
    ));

    // No viewer or Core handle remains to own or restart the adapter.
    drop((first, second, attachment, core));

    let core = sandbox.connect();
    let listed = core.list(false).unwrap();
    let survivor = listed
        .terminals
        .iter()
        .find(|listed| listed.address == terminal.address)
        .expect("the adapter's original terminal survived detaching");
    assert_eq!(survivor.state, TerminalState::Live);
    assert_eq!(survivor.pid, Some(pid));
    assert!(!survivor.keep);
    assert_eq!(
        survivor
            .labels
            .get(sheprd_chat_protocol::TOKEN)
            .map(String::as_str),
        Some(endpoint.as_str())
    );
    let reattached = core
        .attach(&Attach {
            observe: true,
            terminal: &survivor.address,
            ..Attach::default()
        })
        .unwrap();
    assert_eq!(reattached.terminal.pid, Some(pid));
    let mut client = ChatClient::attach(&address);
    let (after_cursor, mut transcript) = snapshot(&mut client);
    assert_eq!(after_cursor.epoch, before_cursor.epoch);
    eprintln!("same adapter epoch after reconnect: {}", after_cursor.epoch);
    assert!(after_cursor.seq > before_cursor.seq);
    assert_eq!(transcript.entries, entries);
    assert_eq!(transcript.session, session);
    wait_for(&mut client, &mut transcript, |transcript| {
        transcript.clients == 1
    });
    complete_turn(&mut client, &mut transcript, "after reattach", "deny");
    assert_eq!(&transcript.entries[..entries.len()], entries.as_slice());
    assert_eq!(transcript.session, session);
    assert!(transcript.entries.iter().any(
        |entry| matches!(&entry.kind, EntryKind::User { text, .. } if text == "after reattach")
    ));

    drop((client, reattached));
    assert_eq!(
        core.kill(&terminal.address).unwrap().state,
        TerminalState::Finished
    );
    drop(core);
    sandbox.stop().unwrap();
    assert!(sandbox.child.try_wait().unwrap().is_some());
    drop(sandbox);
    assert!(!private_dir.exists(), "private sandbox was not removed");
    eprintln!("private terminal killed, Core child reaped, and sandbox removed");
}

fn snapshot(client: &mut ChatClient) -> (Cursor, Box<Transcript>) {
    let ServerMessage::Snapshot { cursor, transcript } = client.next() else {
        panic!("expected the current transcript");
    };
    (cursor, transcript)
}

fn receive(
    client: &mut ChatClient,
    transcript: &mut Transcript,
    deadline: Instant,
) -> ServerMessage {
    let remaining = deadline.saturating_duration_since(Instant::now());
    assert!(
        !remaining.is_zero(),
        "adapter did not reach the expected state"
    );
    client.reader.set_timeout(Some(remaining)).unwrap();
    let message = client.next();
    match &message {
        ServerMessage::Change { change, .. } => transcript.apply(change).unwrap(),
        ServerMessage::Bye { reason } => panic!("adapter closed: {reason:?}"),
        _ => {}
    }
    message
}

fn wait_for(
    client: &mut ChatClient,
    transcript: &mut Transcript,
    ready: impl Fn(&Transcript) -> bool,
) {
    let deadline = Instant::now() + WAIT;
    while !ready(transcript) {
        receive(client, transcript, deadline);
    }
}

fn accepted(client: &mut ChatClient, transcript: &mut Transcript, command: CommandId) {
    let deadline = Instant::now() + WAIT;
    loop {
        if let ServerMessage::Ack { id, rejected } = receive(client, transcript, deadline)
            && id == command
        {
            assert_eq!(rejected, None);
            return;
        }
    }
}

fn complete_turn(client: &mut ChatClient, transcript: &mut Transcript, text: &str, option: &str) {
    let prompt = client.command(ChatCommand::Prompt {
        text: text.into(),
        attachments: Vec::new(),
    });
    accepted(client, transcript, prompt);
    wait_for(client, transcript, |transcript| {
        !transcript.permissions.is_empty()
    });
    let permission = &transcript.permissions[0];
    assert_eq!(permission.title, "Run fixture command");
    let answer = client.command(ChatCommand::Answer {
        option: option.into(),
        permission: permission.id,
    });
    accepted(client, transcript, answer);
    wait_for(client, transcript, |transcript| {
        transcript.turn.is_none()
            && transcript.status == Status::Idle
            && transcript.permissions.is_empty()
    });
    let reply = transcript.entries.iter().rev().find_map(|entry| {
        if let EntryKind::Agent { text } = &entry.kind {
            Some(text.as_str())
        } else {
            None
        }
    });
    assert_eq!(reply, Some(format!("chosen {option}").as_str()));
}
