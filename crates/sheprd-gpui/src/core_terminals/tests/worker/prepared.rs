// ABOUTME: Exercises prepared clipboard input through the real Core worker.
// ABOUTME: Checks output drain, FIFO, cancellation, and bounded pending capacity.
use super::*;

fn start() -> (Scripted, worker::Link) {
    let backend = Scripted::default();
    let link = worker::spawn(
        backend.clone(),
        TerminalId::new("clipboard").unwrap(),
        size(80, 24),
        Role::Active,
    )
    .unwrap();
    until(|| (!backend.received.lock().unwrap().is_empty()).then_some(()));
    (backend, link)
}

fn text(value: &str) -> Command {
    Command::Input(ClientPaneInputEvent::TextCommit(value.into()))
}

#[test]
fn pending_clipboard_keeps_output_live_and_following_input_ordered() {
    let (backend, link) = start();
    let paste = link.prepare().unwrap();
    assert!(link.send(text("following")));
    backend
        .updates
        .lock()
        .unwrap()
        .push(vec![SurfaceUpdate::Title("live output".into())]);
    until(|| {
        link.mailbox
            .take()
            .filter(|screen| screen.title.as_deref() == Some("live output"))
    });
    assert_eq!(
        backend.received.lock().unwrap().as_slice(),
        [Command::Resize(size(80, 24))]
    );
    paste.finish(Some(ClientPaneInputEvent::Paste("clipboard".into())));
    let received = until(|| {
        let received = backend.received.lock().unwrap().clone();
        received
            .iter()
            .any(|command| command == &text("following"))
            .then_some(received)
    });
    assert_eq!(
        received,
        [
            Command::Resize(size(80, 24)),
            Command::Input(ClientPaneInputEvent::Paste("clipboard".into())),
            text("following")
        ]
    );
}

#[test]
fn cancelling_one_reserved_slot_never_flushes_other_accepted_input() {
    let (backend, link) = start();
    // Hold the head so both ordinary inputs remain queued during cancellation.
    let head = link.prepare().unwrap();
    assert!(link.send(text("before")));
    let paste = link.prepare().unwrap();
    assert!(link.send(text("after")));
    assert!(link.send(Command::Resize(size(100, 30))));
    assert!(link.send(Command::Focus(true)));
    backend
        .updates
        .lock()
        .unwrap()
        .push(vec![SurfaceUpdate::Title("output still drains".into())]);
    until(|| {
        link.mailbox
            .take()
            .filter(|screen| screen.title.as_deref() == Some("output still drains"))
    });
    drop(paste);
    head.finish(None);
    let received = until(|| {
        let received = backend.received.lock().unwrap().clone();
        received.contains(&Command::Focus(true)).then_some(received)
    });
    assert_eq!(
        received,
        [
            Command::Resize(size(80, 24)),
            text("before"),
            text("after"),
            Command::Resize(size(100, 30)),
            Command::Focus(true),
        ]
    );
}

#[test]
fn cancelled_clipboard_preserves_waiting_input_size_focus_and_later_input() {
    let (backend, link) = start();
    let paste = link.prepare().unwrap();
    assert!(link.send(text("following")));
    assert!(link.send(Command::Resize(size(100, 30))));
    assert!(link.send(Command::Focus(true)));
    drop(paste);
    assert!(link.send(text("later")));
    let received = until(|| {
        let received = backend.received.lock().unwrap().clone();
        received
            .iter()
            .any(|command| command == &text("later"))
            .then_some(received)
    });
    assert_eq!(
        received,
        [
            Command::Resize(size(80, 24)),
            text("following"),
            Command::Resize(size(100, 30)),
            Command::Focus(true),
            text("later")
        ]
    );
}

#[test]
fn a_waiting_head_keeps_its_capacity_slot_until_completion() {
    let (backend, link) = start();
    let paste = link.prepare().unwrap();
    backend
        .updates
        .lock()
        .unwrap()
        .push(vec![SurfaceUpdate::Title("waiting".into())]);
    until(|| {
        link.mailbox
            .take()
            .filter(|screen| screen.title.as_deref() == Some("waiting"))
    });
    let accepted = (0..1024).take_while(|_| link.send(text("queued"))).count();
    assert!(
        accepted < 1024,
        "pending reads must not unbound the input queue"
    );
    assert!(!link.send(text("overflow")));
    drop(paste);
    until(|| link.send(Command::Resize(size(90, 25))).then_some(()));
    until(|| {
        backend
            .received
            .lock()
            .unwrap()
            .iter()
            .any(|command| command == &Command::Resize(size(90, 25)))
            .then_some(())
    });
    assert_eq!(
        backend
            .received
            .lock()
            .unwrap()
            .iter()
            .filter(|command| matches!(command, Command::Input(_)))
            .count(),
        accepted,
        "cancelling the clipboard slot must preserve every accepted input"
    );
}
