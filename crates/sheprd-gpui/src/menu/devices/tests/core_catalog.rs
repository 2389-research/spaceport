// ABOUTME: Checks Core machine catalog freshness and bounded device rows.
// ABOUTME: Uses explicit Core opt-in before creating the isolated window.
use super::*;
use crate::sidebar::layout_tests::fixture_core_window;
use gpui::{VisualTestContext, px, size};
use sheprd_core_client::wire::{Machine, MachineView};
use sheprd_core_client::{Error as CoreError, Method};

fn machine() -> MachineView {
    MachineView {
        machine: Machine {
            host: "fixture.invalid".into(),
            id: "stable-machine-id".into(),
            name: "lab".into(),
            session: None,
            target: "fixture.invalid".into(),
            temporary: false,
            version: "0.1.18".into(),
            via: false,
        },
        since_ms: None,
        state: "connected".into(),
        why: None,
    }
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn device_picker_shows_only_fresh_core_machine_facts(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.update(|_, cx| {
        crate::core_terminals::set_machine_catalog_for_test(cx, Some(vec![machine()]));
    });
    view.update_in(cx, |view, window, cx| {
        assert!(view.open_menu(window, cx));
        view.menu.page = Some(Page::Devices);
    });
    draw(cx);
    assert!(cx.debug_bounds("core-machine-0").is_some());

    cx.update(|_, cx| {
        let machines = (0..51)
            .map(|index| {
                let mut view = machine();
                view.machine.id = format!("machine-{index:02}");
                view
            })
            .collect();
        crate::core_terminals::set_machine_catalog_for_test(cx, Some(machines));
    });
    draw(cx);
    assert!(cx.debug_bounds("core-machine-49").is_some());
    assert!(cx.debug_bounds("core-machine-50").is_none());
    assert!(cx.debug_bounds("core-machines-more").is_some());

    cx.update(|_, cx| crate::core_terminals::set_machine_catalog_for_test(cx, None));
    draw(cx);
    assert!(cx.debug_bounds("core-machine-0").is_none());
    assert!(cx.debug_bounds("core-machines-unavailable").is_some());
}

#[test]
fn core_machine_labels_have_a_unicode_safe_limit() {
    let long = "🙂".repeat(200);
    let shown = super::super::core_catalog::short(&long);
    assert_eq!(shown.chars().count(), 161);
    assert!(shown.ends_with('…'));
    assert_eq!(super::super::core_catalog::short("lab"), "lab");
}

#[gpui::test]
fn device_picker_bounds_core_refusal_but_keeps_the_typed_cause(cx: &mut gpui::TestAppContext) {
    let long = "🙂".repeat(1_000);
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.update(|_, cx| {
        crate::core_terminals::set_machine_catalog_error_for_test(
            cx,
            CoreError::Refused {
                method: Method::MachineList,
                kind: Some("fixture".into()),
                message: long.clone(),
                hint: None,
            },
        );
    });
    view.update_in(cx, |view, window, cx| {
        assert!(view.open_menu(window, cx));
        view.menu.page = Some(Page::Devices);
    });
    draw(cx);
    assert!(cx.debug_bounds("core-machines-unavailable").is_some());
    let (shown, stored) = cx.update(|_, cx| {
        let error = crate::core_terminals::machine_catalog_error(cx).unwrap();
        (
            super::super::core_catalog::unavailable_reason(Some(error)),
            error.to_string(),
        )
    });
    assert!(shown.chars().count() <= 161);
    assert!(shown.ends_with('…'));
    assert!(stored.ends_with(&long));
}
