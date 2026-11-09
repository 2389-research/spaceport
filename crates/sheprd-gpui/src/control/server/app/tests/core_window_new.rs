use super::*;
use crate::{browser::CoreGroupKey, core_terminals, sidebar::layout_tests::fixture_window};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

#[gpui::test]
fn window_new_targets_the_exact_selected_core_group_without_an_endpoint(
    cx: &mut gpui::TestAppContext,
) {
    let group = Uuid::from_u128(71);
    let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.cx.update(|cx| {
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
    });
    view.update(cx, |view, cx| {
        assert!(view.show_core_group(&key, cx));
        view.endpoints.clear();
        view.live.snapshot = None;
    });
    let named = Caller {
        window: Some(cx.cx.update(|cx| main_windows(cx)[0].window_id().as_u64())),
        ..Caller::default()
    };
    cx.cx.update(|cx| {
        assert_eq!(
            new_window_target(&named, cx).unwrap(),
            WindowLaunch::Core(key.clone())
        );
        assert_eq!(
            new_window_target(&Caller::default(), cx).unwrap(),
            WindowLaunch::Core(key.clone())
        );
        core_terminals::set_document_for_test_as(
            cx,
            "hub-b",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":group,"label":"Another hub"}
                ]}),
                version: 4,
            },
        );
        let wrong_hub = new_window_target(&named, cx).err().unwrap();
        assert_eq!(code(&wrong_hub), Some(ErrorCode::NotFound));
        core_terminals::set_document_stale_for_test(cx);
        let refused = new_window_target(&named, cx).err().unwrap();
        assert_eq!(code(&refused), Some(ErrorCode::NotFound));
    });
}
