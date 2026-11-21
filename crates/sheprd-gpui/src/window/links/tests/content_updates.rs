// ABOUTME: Exercises link clicks across terminal redraws and target changes.
// ABOUTME: Compares opened file identities across native path spellings.
use super::*;
use gpui::MouseButton;
use sheprd_client::{ConnectTarget, protocol::PaneSurfaceScrollMetrics};

/// Short enough to sit on one row of the 20-column fixture, with room left
/// so the row-local detector reads it as a finished address.
const WEB: &str = "https://ex.com/a";
const OTHER: &str = "https://ex.com/b";

/// A pane whose first row is `text` and whose other cells are blank.
fn single_row(text: &str, content_revision: u64) -> Arc<PaneSurfaceFrame> {
    assert!(
        text.chars().count() < 20,
        "{text} fills the row and is not a local link"
    );
    let mut surface = wrapped(content_revision);
    let frame = &mut Arc::make_mut(&mut surface).frame;
    for (cell, symbol) in frame
        .cells
        .iter_mut()
        .zip(text.chars().chain(std::iter::repeat(' ')))
    {
        cell.symbol = symbol.to_string();
        cell.hyperlink = None;
    }
    surface
}

/// `text` as an OSC 8 hyperlink to `target` on the first row.
fn hyperlink(text: &str, target: &str, content_revision: u64) -> Arc<PaneSurfaceFrame> {
    let mut surface = single_row(text, content_revision);
    let columns = text.chars().count();
    let frame = &mut Arc::make_mut(&mut surface).frame;
    frame.hyperlinks = vec![target.to_owned()];
    for cell in frame.cells.iter_mut().take(columns) {
        cell.hyperlink = Some(0);
    }
    surface
}

fn place(view: &mut SheprdWindow, text: &str, revision: u64) {
    view.live.surface = Some(single_row(text, revision));
}

/// Press, replace the surface, release. The replacement is the whole
/// gesture's content change: nothing waits on a timer or a live agent.
fn release_after(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    position: Point<Pixels>,
    modifiers: Modifiers,
    between: impl FnOnce(&mut SheprdWindow),
) {
    cx.simulate_mouse_down(position, MouseButton::Left, modifiers);
    view.update(cx, |view, _| between(view));
    cx.simulate_mouse_up(position, MouseButton::Left, modifiers);
}

fn assert_not_opened(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext, label: &str) {
    assert!(cx.opened_url().is_none(), "{label}");
    view.read_with(cx, |view, _| {
        assert!(view.pressed_terminal_link.is_none(), "{label}");
    });
}

/// A redraw that leaves the address in place completes the click, whichever
/// revision each half of the gesture read.
#[gpui::test]
fn an_unchanged_url_opens_at_the_release(cx: &mut gpui::TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = window(&peer, false, cx);
    let link = at(&view, cx, 2, 0);

    view.update(cx, |view, _| place(view, WEB, 2));
    release_after(&view, cx, link, crate::keymap::secondary_key(), |view| {
        place(view, WEB, 4);
    });
    assert_eq!(cx.opened_url().as_deref(), Some(WEB));

    // The same address at the same revision opens the same way.
    view.update(cx, |view, _| place(view, WEB, 2));
    cx.simulate_click(link, crate::keymap::secondary_key());
    assert_eq!(cx.opened_url().as_deref(), Some(WEB));
}

/// Replacing or clearing the address cancels the click: the release is a
/// different link from the press.
#[gpui::test]
fn a_changed_or_removed_url_does_not_open(cx: &mut gpui::TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = window(&peer, false, cx);
    let link = at(&view, cx, 2, 0);

    view.update(cx, |view, _| place(view, WEB, 2));
    release_after(&view, cx, link, crate::keymap::secondary_key(), |view| {
        place(view, OTHER, 2);
    });
    assert_not_opened(&view, cx, "changed url");
    view.read_with(cx, |view, _| {
        assert_eq!(view.terminal_link_at(link).as_deref(), Some(OTHER));
    });

    view.update(cx, |view, _| place(view, WEB, 2));
    release_after(&view, cx, link, crate::keymap::secondary_key(), |view| {
        place(view, "see notes", 2);
    });
    assert_not_opened(&view, cx, "removed url");
    view.read_with(cx, |view, _| assert!(view.terminal_link_at(link).is_none()));
}

/// A link is what it points at, so a redraw that keeps the address under the
/// pointer keeps the click: another boot, pane, rectangle, or scroll offset
/// is the same link, and so is a revision that only redrew other cells.
#[gpui::test]
fn a_redraw_that_keeps_the_address_keeps_the_click(cx: &mut gpui::TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = window(&peer, false, cx);
    let link = at(&view, cx, 2, 0);

    for move_pane in [
        &(|view: &mut SheprdWindow| {
            let boot = "next-boot";
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id = boot.into();
            Arc::make_mut(view.live.surface.as_mut().unwrap()).boot_id = boot.into();
        }) as &dyn Fn(&mut SheprdWindow),
        &|view: &mut SheprdWindow| {
            Arc::make_mut(view.live.surface.as_mut().unwrap()).panes[0].pane_id = "other".into();
        },
        &|view: &mut SheprdWindow| {
            Arc::make_mut(view.live.surface.as_mut().unwrap()).panes[0]
                .inner_rect
                .height = 3;
        },
        &|view: &mut SheprdWindow| {
            Arc::make_mut(view.live.surface.as_mut().unwrap()).panes[0].scroll =
                Some(PaneSurfaceScrollMetrics {
                    offset_from_bottom: 4,
                    max_offset_from_bottom: 10,
                    viewport_rows: 4,
                });
        },
        &|view: &mut SheprdWindow| {
            Arc::make_mut(view.live.surface.as_mut().unwrap()).panes[0].content_revision = 6;
        },
    ] {
        view.update(cx, |view, _| place(view, WEB, 2));
        release_after(&view, cx, link, crate::keymap::secondary_key(), |view| {
            place(view, WEB, 4);
            move_pane(view);
        });
        assert_eq!(cx.opened_url().as_deref(), Some(WEB));
    }
}

/// Retargeting a device session clears the pending press, so a release on
/// an identical-looking address does not inherit it.
#[gpui::test]
fn endpoint_retargeting_drops_a_pending_press(cx: &mut gpui::TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = window(&peer, false, cx);
    view.update(cx, |view, _| {
        place(view, WEB, 2);
        view.endpoints[0].live.supports_surface = true;
        view.endpoints[0].live.status = ConnectionStatus::Connected;
        let mut remote = crate::endpoint::Endpoint::new(
            "ssh:box".into(),
            "Box".into(),
            ConnectTarget::Ssh {
                target: "user@box".into(),
                session: "default".into(),
            },
            true,
        );
        remote.live.supports_surface = true;
        view.endpoints.push(remote);
    });
    let link = at(&view, cx, 2, 0);
    cx.simulate_mouse_down(link, MouseButton::Left, crate::keymap::secondary_key());
    let (surface, snapshot) = view.read_with(cx, |view, _| {
        assert!(view.pressed_terminal_link.is_some());
        (view.live.surface.clone(), view.live.snapshot.clone())
    });
    view.update(cx, |view, cx| {
        view.select_device_session("ssh:box", "other", cx);
        assert!(view.pressed_terminal_link.is_none());
        view.live.surface = surface;
        view.live.snapshot = snapshot;
        view.live.status = ConnectionStatus::Connected;
        view.live
            .activation
            .transition(crate::state::activation::Transition::Reset(None));
    });
    cx.simulate_mouse_up(link, MouseButton::Left, crate::keymap::secondary_key());
    assert_not_opened(&view, cx, "retarget");
    view.read_with(cx, |view, _| {
        assert_eq!(view.terminal_link_at(link).as_deref(), Some(WEB));
    });
}

/// A printed path survives a redraw the same way. A `file://` hyperlink on
/// another host names its target, so a click says so; a span only the daemon
/// resolved is no click at all, because this client reads no target for it.
#[gpui::test]
fn a_file_link_survives_a_redraw_and_a_resolved_span_is_no_click(cx: &mut gpui::TestAppContext) {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("notes.md"), "# Notes\n").unwrap();
    let path = "notes.md";
    let mut peer = MockPeer::advertising(&["pane.link.resolve"]);
    let (view, cx) = window(&peer, true, cx);
    let modifiers = crate::keymap::secondary_key();

    view.update(cx, |view, _| {
        // The pane prints a path relative to the directory it is in.
        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.panes[0].cwd = Some(folder.path().to_str().unwrap().into());
        snapshot.panes[0].foreground_cwd = snapshot.panes[0].cwd.clone();
        place(view, path, 2);
    });
    let link = at(&view, cx, 2, 0);
    // A printed path is a link once it is found on disk, as a hover with
    // the modifier held would have it.
    view.update(cx, |view, _| {
        let file = view.file_link_at(link).unwrap();
        view.links.checked = Some((file, Some(true)));
    });
    release_after(&view, cx, link, modifiers, |view| place(view, path, 4));
    let opened = cx.opened_url().unwrap();
    let real = folder.path().join(path).canonicalize().unwrap();
    let opened_path = url::Url::parse(&opened).unwrap().to_file_path().unwrap();
    assert_eq!(opened_path.canonicalize().unwrap(), real);
    // Nothing else opens from here on.
    let nothing_more =
        |cx: &mut VisualTestContext| assert_eq!(cx.opened_url(), Some(opened.clone()));

    // A `file://` hyperlink this machine cannot read names its target, so a
    // click says so instead of opening anything.
    let target = "file://devbox/home/dev/notes.md";
    view.update(cx, |view, _| {
        view.live.surface = Some(hyperlink("notes.md", target, 2));
    });
    view.read_with(cx, |view, _| {
        assert!(view.terminal_link_at(link).is_none());
        assert!(view.file_link_at(link).is_none());
        let pressed = view.terminal_link_press(link, modifiers).unwrap();
        assert!(pressed.url.is_none() && pressed.file.is_none());
    });
    cx.simulate_click(link, modifiers);
    nothing_more(cx);
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
    });

    // The daemon's span says these cells are one link; no click follows.
    view.update(cx, |view, _| view.live.surface = Some(wrapped(2)));
    let continuation = at(&view, cx, 2, 1);
    cx.simulate_mouse_move(continuation, None, modifiers);
    resolve_after_delay(cx);
    let request = peer.request();
    assert_eq!(request["method"], "pane.link.resolve");
    answer(
        &mut peer,
        &view,
        cx,
        &request,
        json!({"type": "pane_link_resolved", "regions": [
            {"row": 0, "start_col": 0, "end_col": 19},
            {"row": 1, "start_col": 0, "end_col": 11},
        ]}),
    );
    view.read_with(cx, |view, _| {
        assert!(view.hovered_daemon_link().is_some());
        assert!(view.terminal_link_press(continuation, modifiers).is_none());
    });
    cx.simulate_click(continuation, modifiers);
    nothing_more(cx);
}
