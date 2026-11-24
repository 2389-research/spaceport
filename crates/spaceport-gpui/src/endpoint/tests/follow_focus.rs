//! A focus asked for on a background host takes the window there, and
//! nothing else a background host does may.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, snapshot};
use gpui::{TestAppContext, VisualTestContext};
use spaceport_client::protocol::ClientShellSnapshot;

const HOST: &str = "ssh:homelab";

fn on_tab(tab: &str, revision: u64) -> ClientShellSnapshot {
    let mut snapshot = snapshot(1);
    snapshot.focused_workspace_id = Some("w0".into());
    snapshot.focused_tab_id = Some(tab.into());
    snapshot.revision = revision;
    snapshot
}

fn remote() -> Endpoint {
    let mut endpoint = Endpoint::new(
        HOST.into(),
        "Homelab".into(),
        ConnectTarget::Ssh {
            target: "me@homelab".into(),
            session: "default".into(),
        },
        true,
    );
    endpoint.never_connect();
    endpoint
}

/// Delivers `snapshot` as the worker does and lets the endpoint take it.
fn receive(endpoint: &mut Endpoint, snapshot: ClientShellSnapshot) {
    receive_at(endpoint, snapshot, Instant::now());
}

fn receive_at(endpoint: &mut Endpoint, snapshot: ClientShellSnapshot, now: Instant) {
    endpoint.connection.inbox.lock().unwrap().apply(
        ClientEvent::Snapshot(Arc::new(snapshot)),
        &crate::state::labels(),
    );
    endpoint.poll(now);
}

#[test]
fn a_replaced_connection_forgets_a_focus_it_saw() {
    let mut endpoint = remote();
    receive(&mut endpoint, on_tab("t0", 1));
    receive(&mut endpoint, on_tab("t1", 2));
    endpoint.stop();
    assert!(!endpoint.focus_requested);
}

/// A window on Local holding a background connection to the host.
fn window_beside_host(
    cx: &mut TestAppContext,
) -> (gpui::Entity<SpaceportWindow>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        let mut endpoint = remote();
        receive(&mut endpoint, on_tab("t0", 1));
        view.endpoints.push(endpoint);
        view
    })
}

/// The same window, once the daemon has moved the host to another tab.
fn window_with_focus_asked_for(
    cx: &mut TestAppContext,
) -> (gpui::Entity<SpaceportWindow>, &mut VisualTestContext) {
    let (view, cx) = window_beside_host(cx);
    ask_for_focus(&view, on_tab("t1", 2), cx);
    (view, cx)
}

fn ask_for_focus(
    view: &gpui::Entity<SpaceportWindow>,
    moved: ClientShellSnapshot,
    cx: &mut VisualTestContext,
) {
    view.update(cx, |view, _| {
        receive(&mut view.endpoints[1], moved);
        assert!(view.endpoints[1].focus_requested);
    });
}

fn follow(view: &gpui::Entity<SpaceportWindow>, now: Instant, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let own = window.window_handle().window_id();
        view.update(cx, |view, cx| view.follow_focus(own, now, cx));
    });
}

fn shown(view: &gpui::Entity<SpaceportWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, _| {
        view.endpoints[view.selected_endpoint].id.clone()
    })
}

#[gpui::test]
fn the_window_follows_a_focus_without_touching_the_saved_selection(cx: &mut TestAppContext) {
    let (view, cx) = window_with_focus_asked_for(cx);
    follow(&view, Instant::now(), cx);
    assert_eq!(shown(&view, cx), HOST);
    view.read_with(cx, |view, _| {
        assert_eq!(view.catalog.desired, None);
        assert!(view.catalog.queued_write.is_none());
        assert!(!view.endpoints[1].focus_requested);
    });
}

#[gpui::test]
fn a_host_just_left_answers_the_windows_own_click_without_taking_it_back(cx: &mut TestAppContext) {
    let (view, cx) = window_beside_host(cx);
    let left = Instant::now();
    let later = left + Duration::from_secs(5);
    view.update(cx, |view, _| {
        view.endpoints[1].left_at = Some(left);
        let answer = left + Duration::from_millis(500);
        receive_at(&mut view.endpoints[1], on_tab("t1", 2), answer);
        assert!(!view.endpoints[1].focus_requested);
    });
    follow(&view, later, cx);
    assert_eq!(shown(&view, cx), LOCAL);
    // A later focus is someone asking again.
    view.update(cx, |view, _| {
        receive_at(&mut view.endpoints[1], on_tab("t0", 3), later);
    });
    follow(&view, later, cx);
    assert_eq!(shown(&view, cx), HOST);
}

#[gpui::test]
fn typing_past_the_leaving_guard_does_not_bring_a_click_back(cx: &mut TestAppContext) {
    let (view, cx) = window_beside_host(cx);
    let left = Instant::now();
    let typed = left + Duration::from_secs(3);
    view.update(cx, |view, _| {
        view.endpoints[1].left_at = Some(left);
        let answer = left + Duration::from_millis(500);
        receive_at(&mut view.endpoints[1], on_tab("t1", 2), answer);
        view.last_input = Some(typed);
    });
    follow(&view, typed + Duration::from_millis(500), cx);
    follow(&view, typed + Duration::from_secs(2), cx);
    assert_eq!(shown(&view, cx), LOCAL);
}

#[gpui::test]
fn a_change_of_host_settles_a_focus_held_by_typing(cx: &mut TestAppContext) {
    let (view, cx) = window_beside_host(cx);
    let typed = Instant::now();
    view.update(cx, |view, cx| {
        assert!(view.switch_endpoint(HOST, cx).is_ok());
        view.last_input = Some(typed);
        // A click in the shown host, whose answer the typing holds.
        receive(&mut view.endpoints[1], on_tab("t1", 2));
        assert!(view.endpoints[1].focus_requested);
        assert!(view.switch_endpoint(LOCAL, cx).is_ok());
        assert!(!view.endpoints[1].focus_requested);
    });
    follow(&view, typed + Duration::from_secs(2), cx);
    assert_eq!(shown(&view, cx), LOCAL);
}

#[gpui::test]
fn a_focus_waits_for_the_user_to_stop_typing(cx: &mut TestAppContext) {
    let (view, cx) = window_with_focus_asked_for(cx);
    let typed = Instant::now();
    view.update(cx, |view, _| view.last_input = Some(typed));
    follow(&view, typed + Duration::from_millis(500), cx);
    assert_eq!(shown(&view, cx), LOCAL);
    follow(&view, typed + Duration::from_secs(2), cx);
    assert_eq!(shown(&view, cx), HOST);
}

#[gpui::test]
fn a_focus_waits_for_a_composition_to_end(cx: &mut TestAppContext) {
    let (view, cx) = window_with_focus_asked_for(cx);
    view.update(cx, |view, _| view.marked = "\u{304b}".into());
    follow(&view, Instant::now(), cx);
    assert_eq!(shown(&view, cx), LOCAL);
    view.read_with(cx, |view, _| assert_eq!(view.marked, "\u{304b}"));
    view.update(cx, |view, _| view.marked.clear());
    follow(&view, Instant::now(), cx);
    assert_eq!(shown(&view, cx), HOST);
}

#[gpui::test]
fn an_open_dialog_keeps_the_window_and_the_focus_is_not_kept_for_later(cx: &mut TestAppContext) {
    let (view, cx) = window_with_focus_asked_for(cx);
    view.update(cx, |view, _| {
        view.menu.page = Some(crate::menu::Page::Devices);
    });
    follow(&view, Instant::now(), cx);
    assert_eq!(shown(&view, cx), LOCAL);
    view.update(cx, |view, _| view.menu.page = None);
    follow(&view, Instant::now(), cx);
    assert_eq!(shown(&view, cx), LOCAL);
}

#[gpui::test]
fn the_shown_host_has_nothing_to_follow(cx: &mut TestAppContext) {
    let (view, cx) = window_with_focus_asked_for(cx);
    view.update(cx, |view, cx| {
        assert!(view.switch_endpoint(HOST, cx).is_ok());
        view.endpoints[1].focus_requested = true;
    });
    follow(&view, Instant::now() + Duration::from_secs(5), cx);
    assert_eq!(shown(&view, cx), HOST);
    view.read_with(cx, |view, _| assert!(!view.endpoints[1].focus_requested));
}

#[gpui::test]
fn one_window_follows_and_another_stays_where_it_is(cx: &mut TestAppContext) {
    let mut second_cx = cx.clone();
    let (first, cx) = window_with_focus_asked_for(cx);
    let (second, second_cx) = window_with_focus_asked_for(&mut second_cx);
    follow(&first, Instant::now(), cx);
    assert_eq!(shown(&first, cx), HOST);
    follow(&second, Instant::now(), second_cx);
    assert_eq!(shown(&second, second_cx), LOCAL);
    second.read_with(second_cx, |view, _| {
        assert!(!view.endpoints[1].focus_requested)
    });
}
