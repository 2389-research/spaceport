// ABOUTME: Checks chat writer receipts against real frame and flush behavior.
// ABOUTME: Exercises partial writes without sockets, daemons, or timing guesses.
use super::*;
use crate::session::Session;
use spaceport_chat_protocol::{Command, encode_frame};

struct FailingWriter {
    bytes: Vec<u8>,
    flushes: usize,
    fail_flush: Option<usize>,
    remaining: usize,
}

impl Write for &mut FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }
        let count = bytes.len().min(self.remaining);
        self.bytes.extend_from_slice(&bytes[..count]);
        self.remaining -= count;
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushes += 1;
        if self.fail_flush == Some(self.flushes) {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }
        Ok(())
    }
}

fn hello() -> ClientMessage {
    ClientMessage::Hello {
        client: ClientKind::Gui,
        revision: REVISION,
        since: None,
        version: VERSION,
    }
}

fn command() -> (CommandId, ClientMessage) {
    let id = CommandId::random();
    (
        id,
        ClientMessage::Command {
            command: Command::Cancel,
            id,
        },
    )
}

struct StalledWriter {
    entered: SyncSender<()>,
    release: Receiver<()>,
    writes: usize,
}

impl Write for StalledWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.writes += 1;
        if self.writes == 2 {
            self.entered.send(()).unwrap();
            self.release.recv().unwrap();
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn draining_a_stalled_writer_ends_without_waiting_for_its_peer() {
    let mut session = Session::default();
    let welcome = ServerMessage::Welcome {
        agent: "Fixture".into(),
        pane: None,
        version: VERSION,
    };
    let snapshot = ServerMessage::Snapshot {
        cursor: Cursor::new_epoch(),
        transcript: Box::new(spaceport_chat_protocol::Transcript::new("Fixture")),
    };
    session.receive(welcome.clone()).unwrap();
    session.receive(snapshot.clone()).unwrap();
    session.submit(Command::Cancel).unwrap();
    let command = session.resend().pop().unwrap();
    let ClientMessage::Command { id, .. } = command else {
        panic!("command")
    };
    let (sender, queued) = mpsc::sync_channel(1);
    sender.send(command).unwrap();
    drop(sender);
    let (written, outcomes) = mpsc::sync_channel(2);
    let writer_written = written.clone();
    let progress = Arc::new(WriterProgress::default());
    let writer_progress = progress.clone();
    let (finished, completion) = mpsc::sync_channel(1);
    let (entered, stalled) = mpsc::sync_channel(0);
    let (release, released) = mpsc::sync_channel(0);
    let writer = thread::spawn(move || {
        write_frames(
            StalledWriter {
                entered,
                release: released,
                writes: 0,
            },
            hello(),
            &queued,
            &writer_written,
            &AtomicBool::new(false),
            &writer_progress,
        );
        let _ = finished.try_send(());
    });
    stalled.recv_timeout(Duration::from_secs(5)).unwrap();
    let (ended, end) = mpsc::sync_channel(1);
    let reader = thread::spawn(move || {
        let writer = drain_writer(writer, &completion, &progress, &written);
        ended.send(()).unwrap();
        writer
    });
    let bounded = end.recv_timeout(Duration::from_secs(5)).is_ok();
    let notices = outcomes.try_iter().collect::<Vec<_>>();
    for outcome in &notices {
        session.delivered(*outcome);
    }
    session.lost(&Error::Closed);
    session.begin();
    session.receive(welcome).unwrap();
    session.receive(snapshot).unwrap();
    let replay = session.resend();
    // Release and reap the exact test writer even when the assertion fails.
    release.send(()).unwrap();
    if let Some(writer) = reader.join().unwrap() {
        writer.join().unwrap();
    }
    assert!(
        bounded,
        "draining waited indefinitely for the stalled writer"
    );
    assert_eq!(notices, vec![WriteOutcome::Uncertain(id)]);
    assert!(replay.is_empty(), "the unobserved write must never replay");
    assert!(
        outcomes.try_recv().is_err(),
        "late completion must stay fenced"
    );
}

#[test]
fn completion_publication_wins_the_timeout_handoff_atomically() {
    let id = CommandId::random();
    let progress = Arc::new(WriterProgress::default());
    let (written, outcomes) = mpsc::sync_channel(2);
    assert!(progress.transition(Progress::Begin(id), &written));
    let (finished, completion) = mpsc::sync_channel(0);
    let worker_progress = progress.clone();
    let worker_written = written.clone();
    let writer = thread::spawn(move || {
        assert!(worker_progress.transition(
            Progress::Complete(WriteOutcome::Written(id)),
            &worker_written
        ));
        finished.send(()).unwrap();
    });
    completion.recv_timeout(Duration::from_secs(5)).unwrap();
    progress.transition(Progress::Stop, &written);
    writer.join().unwrap();
    assert_eq!(
        outcomes.try_iter().collect::<Vec<_>>(),
        vec![WriteOutcome::Written(id)]
    );
}

#[test]
fn timeout_publication_fences_late_completion_and_later_commands() {
    let id = CommandId::random();
    let progress = Arc::new(WriterProgress::default());
    let (written, outcomes) = mpsc::sync_channel(2);
    assert!(progress.transition(Progress::Begin(id), &written));
    let (release, released) = mpsc::sync_channel(0);
    let worker_progress = progress.clone();
    let worker_written = written.clone();
    let writer = thread::spawn(move || {
        released.recv().unwrap();
        assert!(!worker_progress.transition(
            Progress::Complete(WriteOutcome::Written(id)),
            &worker_written
        ));
        assert!(!worker_progress.transition(Progress::Begin(CommandId::random()), &worker_written));
    });
    progress.transition(Progress::Stop, &written);
    release.send(()).unwrap();
    writer.join().unwrap();
    assert_eq!(
        outcomes.try_iter().collect::<Vec<_>>(),
        vec![WriteOutcome::Uncertain(id)]
    );
}

#[test]
fn a_partial_command_write_reports_uncertainty_and_leaves_later_commands_unattempted() {
    let (id, command) = command();
    let (_, later) = self::command();
    let hello = hello();
    let hello_bytes = encode_frame(&hello).unwrap();
    let mut writer = FailingWriter {
        bytes: Vec::new(),
        flushes: 0,
        fail_flush: None,
        remaining: hello_bytes.len() + 5,
    };
    let (sender, queued) = mpsc::sync_channel(2);
    sender.send(command).unwrap();
    sender.send(later.clone()).unwrap();
    drop(sender);
    let (written, outcomes) = mpsc::sync_channel(2);
    write_frames(
        &mut writer,
        hello,
        &queued,
        &written,
        &AtomicBool::new(false),
        &WriterProgress::default(),
    );
    assert_eq!(outcomes.try_recv().unwrap(), WriteOutcome::Uncertain(id));
    assert!(outcomes.try_recv().is_err());
    assert_eq!(queued.try_recv().unwrap(), later);
    assert_eq!(&writer.bytes[..hello_bytes.len()], &hello_bytes);
    assert_eq!(writer.bytes.len(), hello_bytes.len() + 5);
}

#[test]
fn a_failed_flush_is_unknown_even_when_the_frame_bytes_were_written() {
    let (id, command) = command();
    let (sender, queued) = mpsc::sync_channel(1);
    sender.send(command.clone()).unwrap();
    drop(sender);
    let (written, outcomes) = mpsc::sync_channel(1);
    let mut writer = FailingWriter {
        bytes: Vec::new(),
        flushes: 0,
        fail_flush: Some(2),
        remaining: usize::MAX,
    };
    write_frames(
        &mut writer,
        hello(),
        &queued,
        &written,
        &AtomicBool::new(false),
        &WriterProgress::default(),
    );
    assert_eq!(outcomes.try_recv().unwrap(), WriteOutcome::Uncertain(id));
    let mut frames = writer.bytes.as_slice();
    assert_eq!(
        read_frame::<_, ClientMessage>(&mut frames).unwrap(),
        Some(hello())
    );
    assert_eq!(
        read_frame::<_, ClientMessage>(&mut frames).unwrap(),
        Some(command)
    );
}

#[test]
fn a_failed_greeting_never_attempts_a_queued_command() {
    let (_, command) = command();
    let (sender, queued) = mpsc::sync_channel(1);
    sender.send(command.clone()).unwrap();
    let (written, outcomes) = mpsc::sync_channel(1);
    let mut writer = FailingWriter {
        bytes: Vec::new(),
        flushes: 0,
        fail_flush: None,
        remaining: 3,
    };
    write_frames(
        &mut writer,
        hello(),
        &queued,
        &written,
        &AtomicBool::new(false),
        &WriterProgress::default(),
    );
    assert!(outcomes.try_recv().is_err());
    assert_eq!(queued.try_recv().unwrap(), command);
    assert_eq!(writer.bytes.len(), 3);
}

#[test]
fn a_complete_flushed_frame_reports_exactly_one_write() {
    let (id, command) = command();
    let (sender, queued) = mpsc::sync_channel(1);
    sender.send(command.clone()).unwrap();
    drop(sender);
    let (written, outcomes) = mpsc::sync_channel(1);
    let mut bytes = Vec::new();
    write_frames(
        &mut bytes,
        hello(),
        &queued,
        &written,
        &AtomicBool::new(false),
        &WriterProgress::default(),
    );
    let mut frames = bytes.as_slice();
    assert_eq!(
        read_frame::<_, ClientMessage>(&mut frames).unwrap(),
        Some(hello())
    );
    assert_eq!(
        read_frame::<_, ClientMessage>(&mut frames).unwrap(),
        Some(command)
    );
    assert!(frames.is_empty());
    assert_eq!(outcomes.try_recv().unwrap(), WriteOutcome::Written(id));
    assert!(outcomes.try_recv().is_err());
}

#[test]
fn a_stopped_writer_does_not_attempt_queued_commands() {
    let (_, command) = command();
    let (sender, queued) = mpsc::sync_channel(1);
    sender.send(command.clone()).unwrap();
    let (written, outcomes) = mpsc::sync_channel(1);
    let mut bytes = Vec::new();
    write_frames(
        &mut bytes,
        hello(),
        &queued,
        &written,
        &AtomicBool::new(true),
        &WriterProgress::default(),
    );
    assert_eq!(queued.try_recv().unwrap(), command);
    assert!(outcomes.try_recv().is_err());
    assert_eq!(bytes, encode_frame(&hello()).unwrap());
}
