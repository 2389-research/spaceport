// ABOUTME: Verifies bounded Ask checkpoint reads and nondestructive New Chat persistence.
// ABOUTME: Keeps these filesystem tests independent of personal agents and daemons.
use super::*;

#[test]
fn new_chat_keeps_the_previous_conversation() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let mut owner = Owner {
        root: root.path().to_owned(),
        broker: root.path().join("unused"),
        conversation: None,
        child: None,
        diagnostics: None,
    };
    owner.new_chat()?;
    let previous = owner.directory()?;
    fs::write(previous.join("saved-history"), "keep")?;
    owner.new_chat()?;
    assert_ne!(owner.directory()?, previous);
    assert_eq!(fs::read_to_string(previous.join("saved-history"))?, "keep");
    assert_eq!(
        fs::read_to_string(root.path().join("current"))?,
        owner
            .conversation
            .map(|id| id.to_string())
            .unwrap_or_default()
    );
    Ok(())
}

#[test]
fn checkpoint_read_is_bounded_and_does_not_hide_corruption() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    assert!(read_checkpoint(root.path())?.is_none());
    fs::write(root.path().join(ask::CHECKPOINT_FILE), b"{")?;
    assert!(matches!(
        read_checkpoint(root.path()),
        Err(Error::Decode(_))
    ));
    fs::write(
        root.path().join(ask::CHECKPOINT_FILE),
        vec![b' '; ask::MAX_CHECKPOINT_BYTES + 1],
    )?;
    assert!(matches!(
        read_checkpoint(root.path()),
        Err(Error::InvalidState)
    ));
    Ok(())
}

#[test]
fn pointer_never_interprets_paths_as_conversation_identity() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("current"), "../../outside")?;
    let mut owner = Owner {
        root: root.path().to_owned(),
        broker: root.path().join("unused"),
        conversation: None,
        child: None,
        diagnostics: None,
    };
    assert!(matches!(owner.restore(), Err(Error::InvalidState)));
    Ok(())
}

#[test]
fn helper_diagnostics_are_bounded_but_fully_drained() {
    let bytes = vec![b'x'; 20_000];
    let mut input = std::io::Cursor::new(bytes);
    assert_eq!(bounded_diagnostic(&mut input).len(), 4096);
    assert_eq!(input.position(), 20_000);
}

#[test]
fn durable_stop_intent_prevents_restart_from_resuming_an_interrupted_checkpoint()
-> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let mut owner = Owner {
        root: root.path().to_owned(),
        broker: root.path().join("unused"),
        conversation: None,
        child: None,
        diagnostics: None,
    };
    owner.new_chat()?;
    let state = Checkpoint {
        version: ask::VERSION,
        conversation_id: owner.conversation.ok_or(Error::InvalidState)?,
        helper_run_id: Uuid::new_v4(),
        endpoint: EndpointId::random().to_string(),
        pi_session: None,
        pi_session_file: None,
        request: Some(ask::Request {
            id: sheprd_chat_protocol::CommandId::random(),
            text: "Original".into(),
            phase: ask::Phase::Interrupted,
        }),
        transcript: sheprd_chat_protocol::Transcript {
            status: sheprd_chat_protocol::Status::Working,
            ..sheprd_chat_protocol::Transcript::new("Pi")
        },
        outcome: Outcome::Interrupted,
    };
    fs::write(
        owner.directory()?.join(ask::CHECKPOINT_FILE),
        serde_json::to_vec(&state)?,
    )?;
    owner.stop_intent()?;
    let restored = owner.restore()?.ok_or(Error::InvalidState)?;
    assert_eq!(restored.outcome, Outcome::Stopped);
    assert_eq!(
        restored.request.map(|request| request.phase),
        Some(ask::Phase::Stopped)
    );
    assert_eq!(
        restored.transcript.status,
        sheprd_chat_protocol::Status::Idle
    );
    assert!(owner.directory()?.join(ask::STOP_FILE).exists());
    Ok(())
}

#[cfg(all(unix, feature = "integration-test"))]
mod shutdown;

#[test]
fn finish_does_not_block_on_a_full_command_queue() -> anyhow::Result<()> {
    let (commands, receiver) = mpsc::sync_channel(1);
    commands.send(Command::Launch)?;
    let (_sender, events) = mpsc::sync_channel(1);
    let (finish, finished) = mpsc::sync_channel(1);
    finish.send(())?;
    let worker = Worker {
        commands,
        events,
        finished,
    };
    let (done, received) = mpsc::sync_channel(1);
    let thread = thread::spawn(move || {
        worker.finish();
        let _ = done.send(());
    });
    let completed = received.recv_timeout(Duration::from_secs(1)).is_ok();
    drop(receiver);
    assert!(thread.join().is_ok(), "finish fixture thread panicked");
    assert!(
        completed,
        "Worker.finish blocked while its command queue was full"
    );
    Ok(())
}
