// ABOUTME: Checks device and session-form departure after active ownership resets.
// ABOUTME: Uses synthetic cached session rows without running discovery or daemon actions.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    menu::sessions::{Edit, Row, Target},
    search_input::SearchInput,
    sidebar::layout_tests::fixture_window,
};
use gpui::{AppContext, ScrollDelta, ScrollEvent, TestAppContext};
use sheprd_client::{ConnectTarget, LocalSession, SessionState};

#[derive(Clone, Copy, Debug)]
enum Content {
    Picker,
    Create,
    Delete,
}

#[gpui::test]
fn devices_and_session_forms_keep_their_body_after_owner_reset(cx: &mut TestAppContext) {
    let mut changed = Vec::new();
    for content in [Content::Picker, Content::Create, Content::Delete] {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = fixture_window(window, cx);
            view.endpoints[0].connection.target =
                ConnectTarget::Socket("/synthetic/no-daemon.sock".into());
            view.sessions.entries = (0..4)
                .map(|index| LocalSession {
                    name: format!("fixture-session-{index}"),
                    state: SessionState::Running,
                    socket: format!("/synthetic/fixture-{index}.sock").into(),
                })
                .collect();
            view.menu.page = Some(Page::Devices);
            view.menu.anchor = point(px(500.), px(650.));
            view.menu.unfolded_device = Some(view.endpoints[0].id.clone());
            view
        });
        cx.simulate_resize(gpui::size(px(800.), px(700.)));
        let field = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.session_edit = match content {
                    Content::Picker => None,
                    Content::Create => Some(Edit::Create {
                        input: cx.new(|cx| {
                            let mut input = SearchInput::new(cx);
                            input.set_text_selected("fixture draft", cx);
                            input
                        }),
                        target: Target::Local,
                    }),
                    Content::Delete => Some(Edit::Delete {
                        row: Row::Local("fixture-session-0".into()),
                        target: Target::Local,
                    }),
                };
                window.focus(&view.menu.focus, cx);
                match &view.menu.session_edit {
                    Some(Edit::Create { input, .. }) => Some(input.downgrade()),
                    _ => None,
                }
            })
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let selector = match content {
            Content::Picker => "sessions-row-3",
            _ => "session-modal",
        };
        let before = cx.debug_bounds(selector).expect("active picker content");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
                view.sessions.entries.clear();
                let later = motion::now() + std::time::Duration::from_secs(3600);
                view.menu.appear.observe(None, None, later);
                let leaving = view.menu.appear.leaving.as_mut().unwrap();
                leaving.motion.set(true, later, true);
                leaving.motion.set(false, later, false);
                assert!(view.menu.session_edit.is_none());
                assert!(view.menu.page.is_none());
            })
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        if cx.debug_bounds(selector) != Some(before) {
            changed.push((content, before, cx.debug_bounds(selector)));
        }
        assert!(field.is_none_or(|field| field.upgrade().is_none()));
    }
    assert!(
        changed.is_empty(),
        "departing picker content changed: {changed:?}"
    );
}

fn leave(view: &mut SheprdWindow, window: &mut Window, cx: &mut Context<SheprdWindow>) {
    view.menu.appear.live = true;
    view.dismiss_menu(window, cx);
    view.sessions.entries.clear();
    view.endpoints[0].label = "replacement observation".into();
    let later = motion::now() + std::time::Duration::from_secs(3600);
    view.menu.appear.observe(None, None, later);
    let leaving = view.menu.appear.leaving.as_mut().unwrap();
    leaving.motion.set(true, later, true);
    leaving.motion.set(false, later, false);
}

#[gpui::test]
fn long_session_rows_keep_their_actual_list_scroll_after_observation_changes(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.endpoints[0].connection.target =
            ConnectTarget::Socket("/synthetic/no-daemon.sock".into());
        view.endpoints[0].label = "long device label 🙂 ".repeat(30);
        view.sessions.entries = (0..60)
            .map(|index| LocalSession {
                name: format!("fixture-session-{index}-{}", "long 🙂 ".repeat(25)),
                state: SessionState::Running,
                socket: format!("/synthetic/fixture-{index}.sock").into(),
            })
            .collect();
        view.menu.page = Some(Page::Devices);
        view.menu.anchor = point(px(20.), px(620.));
        view.menu.unfolded_device = Some(view.endpoints[0].id.clone());
        view
    });
    cx.simulate_resize(gpui::size(px(420.), px(700.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let row = cx.debug_bounds("sessions-row-3").unwrap();
    cx.simulate_event(ScrollEvent {
        position: row.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds("sessions-row-3").unwrap();
    assert_eq!(before.top(), row.top() - px(80.));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| leave(view, window, cx));
        window.draw(cx).clear(cx);
    });
    assert_eq!(cx.debug_bounds("sessions-row-3"), Some(before));
}

#[gpui::test]
fn long_session_errors_keep_their_actual_modal_scroll_when_leaving(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.endpoints[0].connection.target =
            ConnectTarget::Socket("/synthetic/no-daemon.sock".into());
        view.menu.page = Some(Page::Devices);
        view.menu.session_edit = Some(Edit::Create {
            input: cx.new(SearchInput::new),
            target: Target::Local,
        });
        view.menu.error = Some("synthetic bounded failure with long words 🙂 ".repeat(30));
        view
    });
    cx.simulate_resize(gpui::size(px(320.), px(240.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let modal = cx.debug_bounds("session-modal").unwrap();
    cx.simulate_event(ScrollEvent {
        position: point(px(160.), px(120.)),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds("session-modal").unwrap();
    assert_eq!(before.top(), modal.top() - px(80.));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| leave(view, window, cx));
        window.draw(cx).clear(cx);
    });
    assert_eq!(cx.debug_bounds("session-modal"), Some(before));
}
