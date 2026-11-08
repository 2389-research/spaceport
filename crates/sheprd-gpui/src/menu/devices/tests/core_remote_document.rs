use super::*;
use gpui::{ScrollDelta, ScrollEvent, VisualTestContext, point, px, size};
use sheprd_core_client::wire::{Doc, Machine, MachineView};

fn machine(id: &str, name: &str) -> MachineView {
    MachineView {
        machine: Machine {
            host: "fixture.invalid".into(),
            id: id.into(),
            name: name.into(),
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
fn selected_machine_shows_its_remote_groups_without_mixing_hub_groups(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.update(|_, cx| {
        crate::core_terminals::set_machine_catalog_for_test(
            cx,
            Some(vec![machine("stable-a", "lab")]),
        );
    });
    draw(cx);
    let picker = cx.debug_bounds("device-picker").unwrap();
    cx.simulate_click(picker.center(), Default::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).menu.page, Some(Page::Devices)));
    let list = cx.debug_bounds("menu-panel").unwrap();
    cx.simulate_event(ScrollEvent {
        position: list.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        ..Default::default()
    });
    draw(cx);
    let row = cx.debug_bounds("core-machine-0").unwrap();
    assert!(
        list.contains(&row.center()),
        "row is outside picker: {row:?} vs {list:?}"
    );
    cx.simulate_click(row.center(), Default::default());
    assert_eq!(
        cx.update(|_, cx| crate::core_terminals::selected_remote_machine(cx)),
        Some("stable-a".into()),
        "row bounds: {row:?}"
    );
    cx.update(|_, cx| assert_eq!(view.read(cx).menu.page, Some(Page::Devices)));
    draw(cx);
    cx.update(|_, cx| {
        assert!(matches!(
            crate::core_terminals::remote_document_status(cx, "stable-a"),
            Some(crate::core_terminals::RemoteDocumentView::Loading)
        ))
    });
    let panel = cx.debug_bounds("menu-panel").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    draw(cx);
    assert!(cx.debug_bounds("core-remote-groups-loading").is_some());
    let workspaces: Vec<_> = (0..21)
        .map(|index| {
            serde_json::json!({
                "id": uuid::Uuid::from_u128(71 + index).to_string(),
                "label": if index == 0 { "🙂".repeat(200) } else { format!("Remote group {index}") }
            })
        })
        .collect();
    cx.update(|_, cx| {
        crate::core_terminals::set_remote_document_for_test(
            cx,
            "stable-a",
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": workspaces}),
                version: 4,
            },
        );
    });
    draw(cx);
    cx.simulate_resize(size(px(320.), px(700.)));
    draw(cx);
    let group = cx.debug_bounds("core-remote-group-0").unwrap();
    assert!(group.left() >= row.left() && group.right() <= px(320.));
    assert!(cx.debug_bounds("core-remote-group-19").is_some());
    assert!(cx.debug_bounds("core-remote-group-20").is_none());
    assert!(cx.debug_bounds("core-remote-groups-more").is_some());
    assert!(
        cx.debug_bounds("core-workspace-00000000-0000-0000-0000-000000000047")
            .is_none()
    );
}

#[gpui::test]
fn remote_group_error_and_disconnect_do_not_show_old_groups(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        crate::core_terminals::set_machine_catalog_for_test(
            cx,
            Some(vec![machine("stable-a", "lab")]),
        );
        assert!(crate::core_terminals::select_remote_machine(cx, "stable-a"));
        crate::core_terminals::set_remote_document_for_test(
            cx,
            "stable-a",
            Doc {
                value: serde_json::json!({"schema_version": 2, "workspaces": []}),
                version: 2,
            },
        );
    });
    view.update_in(cx, |view, window, cx| {
        assert!(view.open_menu(window, cx));
        view.menu.page = Some(Page::Devices);
    });
    draw(cx);
    assert!(cx.debug_bounds("core-remote-groups-error").is_some());
    cx.update(|_, cx| {
        crate::core_terminals::set_selected_remote_machine_state_for_test(
            cx,
            "stable-a",
            "unreachable",
        );
    });
    draw(cx);
    assert!(cx.debug_bounds("core-remote-groups-stale").is_some());
    assert!(cx.debug_bounds("core-remote-group-0").is_none());
    cx.update(|_, cx| crate::core_terminals::set_machine_catalog_for_test(cx, None));
    draw(cx);
    assert!(cx.debug_bounds("core-remote-groups-stale").is_some());
}
