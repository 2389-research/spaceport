use super::*;
use gpui::{VisualTestContext, px, size};
use sheprd_core_client::wire::{Machine, MachineView};

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
    let (view, cx) = cx.add_window_view(fixture_window);
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
