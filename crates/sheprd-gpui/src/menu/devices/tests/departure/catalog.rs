// ABOUTME: Keeps captured Core machine and group facts after observation replacement.
// ABOUTME: Exercises passive catalog selection without starting a Core connection worker.
use super::*;
use crate::core_terminals;
use gpui::Modifiers;
use sheprd_core_client::wire::{Doc, Machine, MachineView};

fn machine(id: &str) -> MachineView {
    MachineView {
        machine: Machine {
            host: "fixture.invalid".into(),
            id: id.into(),
            name: "long Core machine 🙂 ".repeat(20),
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

#[gpui::test]
fn captured_core_facts_ignore_new_observation_and_cannot_select_a_machine(cx: &mut TestAppContext) {
    let (root, cx) = cx.add_window_view(|window, cx| Preview {
        source: cx.new(|cx| {
            crate::core_feature::install(true, cx);
            let mut view = fixture(window, cx);
            view.menu.unfolded_device = None;
            core_terminals::set_machine_catalog_for_test(cx, Some(vec![machine("old")]));
            assert!(core_terminals::select_remote_machine(cx, "old"));
            core_terminals::set_remote_document_for_test(
                cx,
                "old",
                Doc {
                    value: serde_json::json!({"schema_version":1,"workspaces":[{
                        "id": uuid::Uuid::from_u128(1).to_string(),
                        "label": "🙂".repeat(200)
                    }]}),
                    version: 1,
                },
            );
            assert!(view.core_enabled());
            assert!(matches!(
                core_terminals::remote_document_status(cx, "old"),
                Some(core_terminals::RemoteDocumentView::Ready(_))
            ));
            view
        }),
        paint: None,
        modal: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds("core-remote-group-0").unwrap();
    cx.update(|window, cx| {
        root.update(cx, |root, cx| {
            root.paint = root.source.update(cx, |view, cx| {
                let paint = view.devices_paint(cx);
                view.dismiss_menu(window, cx);
                core_terminals::set_machine_catalog_for_test(
                    cx,
                    Some(vec![machine("old"), machine("new")]),
                );
                assert!(core_terminals::select_remote_machine(cx, "new"));
                paint
            });
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    assert_eq!(cx.debug_bounds("core-remote-group-0"), Some(before));
    let row = cx.debug_bounds("core-machine-0").unwrap();
    assert!(
        cx.debug_bounds("devices-list")
            .unwrap()
            .contains(&row.center())
    );
    cx.simulate_mouse_move(row.center(), None, Modifiers::default());
    cx.simulate_click(row.center(), Modifiers::default());
    cx.update(|_, cx| {
        assert_eq!(
            core_terminals::selected_remote_machine(cx).as_deref(),
            Some("new")
        )
    });
}
