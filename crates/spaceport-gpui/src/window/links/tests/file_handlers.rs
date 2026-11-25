//! A `file://` hyperlink opens in the system's application, read from the row
//! itself: no click reaches the daemon, so no link handler can claim it. What
//! this machine cannot open, such as another host's file, a click explains.

use super::*;

/// A 20-column pane whose first row is an OSC 8 hyperlink to `target`.
pub(super) fn hyperlinked(target: &str) -> Arc<PaneSurfaceFrame> {
    let mut surface = wrapped(2);
    let frame = &mut Arc::make_mut(&mut surface).frame;
    frame.hyperlinks = vec![target.to_owned()];
    for (cell, symbol) in frame
        .cells
        .iter_mut()
        .zip("notes.md".chars().chain(std::iter::repeat(' ')))
    {
        cell.symbol = symbol.to_string();
        cell.hyperlink = None;
    }
    for cell in &mut frame.cells[..8] {
        cell.hyperlink = Some(0);
    }
    surface
}

/// Shows a hyperlink to `document`, whose file opens in the system's
/// application, which these tests observe.
fn show(view: &mut SpaceportWindow, document: &Document) {
    view.live.surface = Some(hyperlinked(&document.url()));
}

/// A Markdown file in a directory of its own, removed when dropped.
struct Document(std::path::PathBuf);

impl Document {
    fn new() -> Self {
        static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "hg-file-handler-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("notes.md"), "# Notes\n").unwrap();
        Self(directory)
    }

    fn url(&self) -> String {
        url::Url::from_file_path(self.0.join("notes.md"))
            .unwrap()
            .into()
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A link-modifier click on a `file://` hyperlink opens the file here at
/// once, with nothing asked of the daemon.
#[gpui::test]
fn a_file_hyperlink_opens_here_without_asking_the_daemon(cx: &mut gpui::TestAppContext) {
    let document = Document::new();
    let peer = MockPeer::advertising(&["pane.link.resolve"]);
    let (view, cx) = window(&peer, true, cx);
    view.update(cx, |view, _| show(view, &document));
    let link = at(&view, cx, 3, 0);
    view.read_with(cx, |view, _| {
        assert!(view.terminal_link_at(link).is_none(), "not a web link");
        assert!(view.hovered_daemon_link().is_none());
    });

    // The click opens the file and queues no request; only a hover would.
    cx.simulate_click(link, spaceport_keymap::secondary_key());
    let opened = cx.opened_url().unwrap();
    assert!(
        opened.starts_with("file://") && opened.ends_with("/notes.md"),
        "{opened}"
    );
    view.read_with(cx, |view, _| {
        assert!(view.links.resolving.is_none());
        assert!(view.pressed_terminal_link.is_none());
    });
}

/// A hover that already resolved the hyperlink changes nothing: the click
/// still opens the file here, from the row's own reading.
#[gpui::test]
fn a_resolved_file_hyperlink_still_opens_here(cx: &mut gpui::TestAppContext) {
    let document = Document::new();
    let mut peer = MockPeer::advertising(&["pane.link.resolve"]);
    let (view, cx) = window(&peer, true, cx);
    view.update(cx, |view, _| show(view, &document));
    let link = at(&view, cx, 3, 0);
    cx.simulate_mouse_move(link, None, spaceport_keymap::secondary_key());
    resolve_after_delay(cx);
    let request = peer.request();
    assert_eq!(request["method"], "pane.link.resolve");
    answer(
        &mut peer,
        &view,
        cx,
        &request,
        json!({"type": "pane_link_resolved", "regions": [
            {"row": 0, "start_col": 0, "end_col": 7},
        ]}),
    );
    view.read_with(cx, |view, _| assert!(view.hovered_daemon_link().is_some()));

    cx.simulate_click(link, spaceport_keymap::secondary_key());
    let opened = cx.opened_url().unwrap();
    assert!(opened.ends_with("/notes.md"), "{opened}");
    view.read_with(cx, |view, _| assert!(view.links.resolving.is_none()));
}

/// A pane on an SSH host prints that host's `file://` links, which never open
/// here. A click claims the gesture and says why, rather than sending it on
/// for a handler on that host to run.
#[gpui::test]
fn a_remote_file_hyperlink_says_why_it_opens_nothing(cx: &mut gpui::TestAppContext) {
    let peer = MockPeer::advertising(&["pane.link.resolve"]);
    let (view, cx) = window(&peer, true, cx);
    let url = "file://devbox/home/dev/notes.md";
    view.update(cx, |view, _| {
        view.live.surface = Some(hyperlinked(url));
        let selected = view.selected_endpoint;
        view.endpoints[selected].connection.target = spaceport_client::ConnectTarget::Ssh {
            target: "devbox.invalid".into(),
            session: "default".into(),
        };
        assert!(view.selected_is_remote());
    });
    let link = at(&view, cx, 3, 0);
    view.read_with(cx, |view, _| {
        assert!(view.file_link_at(link).is_none(), "not this machine's file");
        assert!(view.hovered_daemon_link().is_none());
    });

    cx.simulate_click(link, spaceport_keymap::secondary_key());
    assert!(cx.opened_url().is_none());
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.flash
                .as_ref()
                .map(|(flash, _)| flash.text.to_string())
                .as_deref(),
            Some(
                crate::Error::LinkOnAnotherHost {
                    host: "devbox".into()
                }
                .to_string()
                .as_str()
            )
        );
        assert!(view.links.resolving.is_none());
    });
    // Without the modifier the click stays the pane's, as on any hyperlink
    // this client cannot open.
    view.read_with(cx, |view, _| {
        assert!(
            view.terminal_link_press(link, Modifiers::default())
                .is_none()
        );
    });
}
