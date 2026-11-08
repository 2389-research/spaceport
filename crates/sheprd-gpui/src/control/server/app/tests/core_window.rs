use crate::{browser::CoreGroupKey, core_terminals, sidebar::layout_tests::fixture_window};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

#[gpui::test]
fn window_list_names_the_selected_core_group_without_a_legacy_workspace(
    cx: &mut gpui::TestAppContext,
) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, _| {
            let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            snapshot.focused_workspace_id = Some("w_legacy".into());
        });
        let legacy = view.read(cx).control_info(window, cx);
        assert_eq!(legacy.workspace.as_deref(), Some("w_legacy"));

        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"North"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
            assert!(view.show_core_group(&key, cx));
        });
        let selected = view.read(cx).control_info(window, cx);
        assert_eq!(selected.workspace, None);
        let json = serde_json::to_value(&selected).unwrap();
        assert_eq!(
            json["core_workspace"],
            serde_json::json!({
                "hub_core_id":"hub-a",
                "machine_id":null,
                "document_name":"spaceport",
                "group_id":group.to_string(),
                "current":true
            })
        );
        core_terminals::set_document_stale_for_test(cx);
        let stale = serde_json::to_value(view.read(cx).control_info(window, cx)).unwrap();
        assert_eq!(stale["core_workspace"]["current"], false);
    });
}
