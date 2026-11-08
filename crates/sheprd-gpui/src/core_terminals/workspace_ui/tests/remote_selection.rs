use super::*;
use crate::endpoint::Endpoint;
use crate::i18n::fl;
use sheprd_client::ConnectTarget;

#[gpui::test]
fn local_core_group_new_tab_ignores_legacy_remote_selection(cx: &mut gpui::TestAppContext) {
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        observation::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": group, "label": "Local"}
                ]}),
                version: 1,
            },
        );
        view.update(cx, |view, cx| {
            view.endpoints.push(Endpoint::new(
                "ssh:fixture".into(),
                "Fixture".into(),
                ConnectTarget::Ssh {
                    target: "fixture".into(),
                    session: "default".into(),
                },
                true,
            ));
            view.selected_endpoint = 1;
            assert!(view.selected_is_remote());
            view.select_core_workspace(group, cx);
            assert!(view.new_tabs_use_core());
            view.new_core_terminal(window, cx);
            assert_eq!(
                view.flash.as_ref().map(|(flash, _)| flash.text.as_str()),
                Some(fl!("core-workspace-stale").as_str())
            );
            view.leave_core_group(cx);
            view.new_core_terminal(window, cx);
            assert_eq!(
                view.flash.as_ref().map(|(flash, _)| flash.text.as_str()),
                Some(fl!("core-terminals-local-only").as_str())
            );
        });
    });
}
