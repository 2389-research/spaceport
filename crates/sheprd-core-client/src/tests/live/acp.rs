//! A real host adapter inside private Core, reached through its local attach socket.
//! The deterministic ACP child exercises the protocol boundary, not native GUI parity.

use super::*;
use sheprd_chat_protocol::{
    Change, ClientKind, ClientMessage, Command as ChatCommand, CommandId, EndpointId, EntryKind,
    PermissionId, Rejection, ServerMessage, VERSION, read_frame, write_frame,
};
use sheprd_client::private_endpoint::{Connection, Reader, Writer};
use std::path::PathBuf;

mod detach;

const WAIT: Duration = Duration::from_secs(10);

struct ChatClient {
    reader: Reader,
    writer: Writer,
}

impl ChatClient {
    fn attach(address: &Path) -> Self {
        let (mut reader, mut writer, _closer) =
            Connection::connect(address, WAIT).unwrap().split().unwrap();
        reader.set_timeout(Some(WAIT)).unwrap();
        write_frame(
            &mut writer,
            &ClientMessage::Hello {
                client: ClientKind::Gui,
                revision: sheprd_chat_protocol::REVISION,
                since: None,
                version: VERSION,
            },
        )
        .unwrap();
        assert!(matches!(
            read_frame::<_, ServerMessage>(&mut reader).unwrap(),
            Some(ServerMessage::Welcome { .. })
        ));
        Self { reader, writer }
    }

    fn next(&mut self) -> ServerMessage {
        read_frame(&mut self.reader)
            .unwrap()
            .expect("adapter closed")
    }

    fn command(&mut self, command: ChatCommand) -> CommandId {
        let id = CommandId::random();
        write_frame(&mut self.writer, &ClientMessage::Command { command, id }).unwrap();
        id
    }
}

#[test]
#[ignore = "needs HERDR_CORE_TEST_BINARY and SHEPRD_ACP_TEST_BINARY"]
fn a_core_owned_adapter_shares_one_permission_result() {
    let core_binary = std::env::var_os("HERDR_CORE_TEST_BINARY").expect("HERDR_CORE_TEST_BINARY");
    let adapter_binary =
        PathBuf::from(std::env::var_os("SHEPRD_ACP_TEST_BINARY").expect("SHEPRD_ACP_TEST_BINARY"));
    assert!(adapter_binary.is_absolute() && adapter_binary.is_file());
    let mut sandbox = Sandbox::start(core_binary);
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
            name: Some("private-acp-peer"),
            ..Run::default()
        })
        .unwrap();
    let listed = core.terminal_get(&terminal.address).unwrap();
    assert_eq!(
        listed
            .labels
            .get(sheprd_chat_protocol::TOKEN)
            .map(String::as_str),
        Some(endpoint.as_str())
    );
    let address = sandbox
        ._dir
        .path()
        .join("state/herdr/acp")
        .join(format!("{endpoint}.sock"));
    let deadline = Instant::now() + WAIT;
    while !address.exists() {
        assert!(
            Instant::now() < deadline,
            "Core-owned adapter did not open private socket"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let mut first = ChatClient::attach(&address);
    assert!(matches!(first.next(), ServerMessage::Snapshot { .. }));
    let prompt = first.command(ChatCommand::Prompt {
        text: "fixture turn".into(),
        attachments: Vec::new(),
    });
    let mut permission = None;
    while permission.is_none() {
        match first.next() {
            ServerMessage::Ack { id, rejected } if id == prompt => assert_eq!(rejected, None),
            ServerMessage::Change {
                change: Change::PermissionAsked { permission: asked },
                ..
            } => {
                assert_eq!(asked.title, "Run fixture command");
                permission = Some(asked.id);
            }
            _ => {}
        }
    }
    let permission: PermissionId = permission.unwrap();
    let mut second = ChatClient::attach(&address);
    assert!(
        matches!(second.next(), ServerMessage::Snapshot { transcript, .. } if transcript.permissions.iter().any(|asked| asked.id == permission))
    );
    let answered = first.command(ChatCommand::Answer {
        option: "allow".into(),
        permission,
    });
    loop {
        if let ServerMessage::Ack { id, rejected } = first.next()
            && id == answered
        {
            assert_eq!(rejected, None);
            break;
        }
    }
    let stale = second.command(ChatCommand::Answer {
        option: "deny".into(),
        permission,
    });
    loop {
        if let ServerMessage::Ack { id, rejected } = second.next()
            && id == stale
        {
            assert_eq!(rejected, Some(Rejection::UnknownPermission));
            break;
        }
    }
    let mut latest = ChatClient::attach(&address);
    let ServerMessage::Snapshot { mut transcript, .. } = latest.next() else {
        panic!("expected the current transcript");
    };
    loop {
        if let Some(text) = transcript
            .entries
            .iter()
            .find_map(|entry| match &entry.kind {
                EntryKind::Agent { text } => Some(text.as_str()),
                _ => None,
            })
        {
            assert_eq!(text, "chosen allow");
            assert!(transcript.permissions.is_empty());
            break;
        }
        if let ServerMessage::Change { change, .. } = latest.next() {
            transcript.apply(&change).unwrap();
        }
    }
    core.kill(&terminal.address).unwrap();
    sandbox.stop().unwrap();
}
