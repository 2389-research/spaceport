// ABOUTME: Checks shortcut reference departure after its live input owners reset.
// ABOUTME: Preserves filtered command rows and an idle editor without keeping search focus.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{controls::COMMANDS, sidebar::layout_tests::fixture_window, state::ConnectionStatus};
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_client::protocol::{ClientShellCommand, ClientShellCommandAction};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

fn command(description: &str) -> ClientShellCommand {
    ClientShellCommand {
        command_id: "fixture.audit".into(),
        description: Some(description.into()),
        action: ClientShellCommandAction::Shell,
        binding_label: "ctrl+alt+a".into(),
        binding_labels: vec!["ctrl+alt+a".into()],
    }
}

fn install(view: &Entity<SheprdWindow>, description: &str, cx: &mut VisualTestContext) {
    view.update(cx, |view, _| {
        std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands =
            vec![command(description)];
        view.live.status = ConnectionStatus::Connected;
    });
}

fn close(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().unwrap();
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            assert!(view.menu.page.is_none());
            assert!(view.menu.keybinds_search.is_none());
            assert!(view.menu.shortcut_editor.is_none());
        });
    });
}

#[gpui::test]
fn filtered_shortcuts_leave_without_following_new_commands_or_retaining_search(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(fixture_window);
    install(&view, "Original review", cx);
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_keybinds(window, cx)));
    let search = view.update(cx, |view, cx| {
        let search = view.menu.keybinds_search.as_ref().unwrap();
        search.update(cx, |input, cx| input.set_text_selected("original", cx));
        search.downgrade()
    });
    draw(cx);
    assert!(cx.debug_bounds("shortcut-Original review").is_some());
    close(&view, cx);
    install(&view, "Replacement review", cx);
    draw(cx);
    assert!(
        search.upgrade().is_none(),
        "the search owner must be released"
    );
    assert!(cx.debug_bounds("shortcut-Original review").is_some());
    assert!(cx.debug_bounds("shortcut-Replacement review").is_none());
}

#[gpui::test]
fn the_shortcut_editor_keeps_its_display_after_its_live_owner_resets(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_keybinds(window, cx);
            let info = COMMANDS.iter().find(|info| info.name == "new_tab").unwrap();
            view.edit_shortcut(info, window, cx);
        });
    });
    draw(cx);
    assert!(cx.debug_bounds("shortcut-editor").is_some());
    close(&view, cx);
    draw(cx);
    assert!(cx.debug_bounds("shortcut-editor").is_some());
}

#[gpui::test]
fn scrolled_shortcuts_leave_from_their_captured_offset(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_keybinds(window, cx)));
    draw(cx);
    let body = cx.debug_bounds("keybinds-body").unwrap();
    let before = cx.debug_bounds("shortcut-New Workspace").unwrap();
    assert!(
        before.top() >= body.top() && before.bottom() <= body.bottom(),
        "row {before:?}; body {body:?}"
    );
    cx.simulate_event(gpui::ScrollEvent {
        position: body.center(),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("shortcut-New Workspace").unwrap();
    assert!(
        scrolled.top() < before.top(),
        "the live reference really scrolls"
    );
    close(&view, cx);
    view.update(cx, |view, _| {
        view.menu.keybinds_scroll = gpui::ScrollHandle::new();
    });
    draw(cx);
    let leaving = cx.debug_bounds("shortcut-New Workspace").unwrap();
    assert!((leaving.top() - scrolled.top()).abs() < px(1.));
    assert_eq!(cx.debug_bounds("keybinds-body").unwrap(), body);
}
