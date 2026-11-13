// ABOUTME: Checks that a selected Core group owns its connection status UI.
// ABOUTME: Keeps a legacy endpoint outage from covering the Core workspace.

use crate::{browser::CoreGroupKey, core_terminals};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

#[gpui::test]
fn core_group_does_not_show_the_legacy_reconnect_overlay(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Core group"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.endpoints[0].outage = Some("legacy connection lost".into());
            assert!(view.reconnecting());

            let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
            assert!(view.show_core_group(&key, cx));
            assert!(!view.reconnecting());

            view.leave_core_group(cx);
            assert!(view.reconnecting());
        });
    });
}
