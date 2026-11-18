// ABOUTME: Holds Core attachment to saturate its actual bounded command channel.
// ABOUTME: Requires rejected input to reach the visible screen mailbox.
use super::*;
use std::sync::mpsc;

struct Paused {
    attaching: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl Backend for Paused {
    type Error = Failure;
    type Terminal = ScriptedTerminal;

    fn attach(&self, _: &TerminalId, _: Size, _: Role) -> Result<Self::Terminal, Failure> {
        self.attaching.send(()).unwrap();
        self.release.lock().unwrap().recv().unwrap();
        Ok(ScriptedTerminal(Scripted::default()))
    }
}

#[test]
fn saturated_input_queue_reports_rejected_input_to_the_view() {
    let (attaching, attached) = mpsc::sync_channel(0);
    let (release, released) = mpsc::sync_channel(0);
    let backend = Paused {
        attaching,
        release: Mutex::new(released),
    };
    let link = worker::spawn(
        backend,
        TerminalId::new("backpressure").unwrap(),
        size(80, 24),
        Role::Active,
    )
    .unwrap();
    attached.recv_timeout(Duration::from_secs(5)).unwrap();
    // More commands than the bounded queue holds; no worker can consume them yet.
    for _ in 0..1024 {
        link.send(Command::Input(ClientPaneInputEvent::TextCommit("x".into())));
    }
    let notice = link.mailbox.take().and_then(|screen| screen.failure);
    drop(link);
    release.send(()).unwrap();
    assert!(
        notice.is_some(),
        "rejected input must produce a visible failure"
    );
}

#[test]
fn draining_a_saturated_queue_wakes_pending_view_changes_without_output() {
    let (attaching, attached) = mpsc::sync_channel(0);
    let (release, released) = mpsc::sync_channel(0);
    let backend = Paused {
        attaching,
        release: Mutex::new(released),
    };
    let link = worker::spawn(
        backend,
        TerminalId::new("retry-view").unwrap(),
        size(80, 24),
        Role::Active,
    )
    .unwrap();
    attached.recv_timeout(Duration::from_secs(5)).unwrap();
    for _ in 0..1024 {
        link.send(Command::Resize(size(100, 30)));
    }
    link.mailbox.take();
    release.send(()).unwrap();
    let screen = until(|| link.mailbox.take());
    assert!(
        screen.frame.is_none(),
        "the retry wakeup must not need terminal output"
    );
}
