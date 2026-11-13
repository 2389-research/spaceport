// ABOUTME: Exercises a selected Core group's full window tick without legacy endpoints.
// ABOUTME: Uses a private join callback so no personal Core can be contacted or started.

use crate::{
    browser::CoreGroupKey,
    core_terminals::{self, Error, Start},
};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn private_join(start: Start) -> Result<sheprd_core_client::CoreBackend, Error> {
    assert_eq!(start, Start::Join);
    Err(Error::NotInstalled)
}

#[gpui::test]
fn selected_core_group_ticks_without_legacy_endpoints(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
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
            let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
            view.core_terminals.hub.set_join_for_test(private_join);
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
            view.tick(window, cx);
            assert!(view.endpoints.is_empty());
        });
    });
}
