// ABOUTME: Checks OS window titles for selected Core presentation groups.
// ABOUTME: Verifies stale Core metadata cannot leak an old title.

use super::*;
use crate::{browser::CoreGroupKey, core_terminals};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

#[gpui::test]
fn selected_core_group_sets_title_from_fresh_exact_document(cx: &mut gpui::TestAppContext) {
    let first = Uuid::from_u128(71);
    let second = Uuid::from_u128(72);
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        core_terminals::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":first,"label":"North"},{"id":second,"label":"South"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.live.window_title = Some("Legacy agent title".into());
            for (id, label) in [(first, "North"), (second, "South")] {
                let key = CoreGroupKey::new("hub-a", None, "spaceport", id).unwrap();
                assert!(view.show_core_group(&key, cx));
                view.sync_window_title(window, cx);
                assert_eq!(view.title, format!("{WINDOW_TITLE} \u{2014} {label}"));
            }
        });
        core_terminals::set_document_stale_for_test(cx);
        view.update(cx, |view, cx| {
            view.sync_window_title(window, cx);
            assert_eq!(view.title, WINDOW_TITLE);
        });
        core_terminals::set_document_for_test_as(
            cx,
            "hub-b",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":second,"label":"Wrong hub"}
                ]}),
                version: 4,
            },
        );
        view.update(cx, |view, cx| {
            view.sync_window_title(window, cx);
            assert_eq!(view.title, WINDOW_TITLE);
            view.leave_core_group(cx);
            view.sync_window_title(window, cx);
            assert_eq!(view.title, "Legacy agent title");
        });
    });
}
