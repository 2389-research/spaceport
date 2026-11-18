// ABOUTME: Exercises local palette commands without a legacy runtime connection.
// ABOUTME: Keeps stale Core ownership from activating a captured popup action.
use super::*;
use crate::{browser::CoreGroupKey, core_terminals, sidebar::layout_tests::fixture_core_window};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn core_window(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            let group = Uuid::from_u128(902);
            let key = CoreGroupKey::new("palette-core", None, "spaceport", group).unwrap();
            core_terminals::set_document_for_test_as(
                cx,
                "palette-core",
                Doc {
                    value: serde_json::json!({"schema_version":1,"workspaces":[
                        {"id":group,"label":"Core palette"}
                    ]}),
                    version: 1,
                },
            );
            assert!(view.show_core_group(&key, cx));
            view.endpoints.clear();
            view.live.snapshot = None;
        })
    });
    (view, cx)
}

#[gpui::test]
fn native_palette_actions_use_the_core_only_popup_owner(cx: &mut TestAppContext) {
    for stale in ["current", "epoch", "group"] {
        let (view, cx) = core_window(cx);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open_palette(Filter::Commands, window, cx);
                assert!(!view.menu_target_current());
                let before = view.sidebar_visible;
                if stale == "epoch" {
                    view.selection_epoch += 1;
                } else if stale == "group" {
                    let group = Uuid::from_u128(903);
                    core_terminals::set_document_for_test_as(
                        cx,
                        "palette-core",
                        Doc {
                            value: serde_json::json!({"schema_version":1,"workspaces":[
                                {"id":group,"label":"Other Core group"}
                            ]}),
                            version: 2,
                        },
                    );
                    let key = CoreGroupKey::new("palette-core", None, "spaceport", group).unwrap();
                    assert!(view.show_core_group(&key, cx));
                }
                view.activate_palette(Action::Native(Command::ToggleSidebar), window, cx);
                assert_eq!(
                    view.sidebar_visible,
                    if stale == "current" { !before } else { before }
                );
                assert_eq!(view.menu.page.is_some(), stale != "current");
            })
        });
    }
}

#[gpui::test]
fn legacy_actions_refuse_in_a_core_only_palette(cx: &mut TestAppContext) {
    for command in [Command::Detach, Command::Reconnect, Command::Workspace] {
        let (view, cx) = core_window(cx);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open_palette(Filter::Commands, window, cx);
                view.activate_palette(Action::Native(command), window, cx);
                assert!(view.menu.page.is_none());
                assert!(
                    view.notices.history().next_back().is_some(),
                    "{command:?} needs a visible refusal"
                );
            })
        });
    }
}

#[gpui::test]
fn daemon_reload_has_no_target_in_a_core_only_window(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    view.update(cx, |view, _| {
        view.endpoints.clear();
        view.reload_daemon_config();
        assert!(view.endpoints.is_empty());
    });
}
