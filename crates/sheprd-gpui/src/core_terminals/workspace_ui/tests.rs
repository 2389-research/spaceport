// ABOUTME: Checks workspace ui behavior with isolated application fixtures.
// ABOUTME: Makes Core opt-in explicit while preserving retained behavior checks.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{Access, EditGuard, GuardState, VersionedDocument, observation};
use crate::{menu::Page, sidebar};
use sheprd_core_client::{Error, Method, wire::Doc};
use uuid::Uuid;

#[gpui::test]
fn core_only_group_switching_keeps_exact_tabs_and_picks(cx: &mut gpui::TestAppContext) {
    use crate::browser::{CoreGroupKey, Pick, Shown, Store};
    use sheprd_chat_protocol::Agent;
    let first = Uuid::from_u128(71);
    let second = Uuid::from_u128(72);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observation::set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[
                {"id":first,"label":"Same"},{"id":second,"label":"Same"}
            ]}), version: 3,
        });
        let key = CoreGroupKey::new("hub-a", None, "spaceport", first).unwrap();
        let tab = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        view.update(cx, |view, cx| {
            view.config.herdr_2_terminals = false;
            view.select_core_workspace(first, cx);
            assert!(view.new_tabs_use_core());
            assert_eq!(view.browser_tab_ids(cx), vec![tab]);
            assert_eq!(view.open_workspace_tabs(cx), vec![Pick::Page(tab)]);
            let group = view.ensure_layout().unwrap().active();
            view.browser.layouts.get_mut(&key.parts()).unwrap().choose(group, Pick::Page(tab));
            assert_eq!(view.group_shown(group, cx), Shown::Page(tab));
            view.start_chat_in(Agent::Opencode, group, cx);
            assert!(view.flash.as_ref().is_some_and(|(flash, _)| flash.text.contains("Core terminal tab")));
            view.select_core_workspace(second, cx);
            assert!(view.browser_tab_ids(cx).is_empty());
            view.live.snapshot = Some(std::sync::Arc::new(sidebar::layout_tests::snapshot(1)));
            let empty_group = view.ensure_layout().unwrap().active();
            view.split_group(empty_group, window, cx);
            assert!(view.group_slots().iter().all(|slot| !matches!(view.group_pick(slot.id), Some(Pick::Herdr(_)))));
            view.select_core_workspace(first, cx);
            assert_eq!(view.group_shown(group, cx), Shown::Page(tab));
            view.go_to_workspace(0, "w0", cx);
            assert!(view.selected_core_group().is_none());
            view.select_core_workspace(first, cx);
        });
        observation::set_document_stale_for_test(cx);
        view.update(cx, |view, cx| {
            assert!(view.browser_tab_ids(cx).is_empty());
            assert_eq!(view.group_shown(view.group_slots()[0].id, cx), Shown::Empty);
        });
        observation::set_document_for_test_as(cx, "hub-b", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":first,"label":"Same"}]}), version: 4,
        });
        view.update(cx, |view, cx| assert!(view.browser_tab_ids(cx).is_empty()));
        observation::set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[]}), version: 5,
        });
        view.update(cx, |view, cx| assert!(view.browser_tab_ids(cx).is_empty()));
    });
}

#[gpui::test]
fn a_closed_core_page_reopens_in_its_exact_group(cx: &mut gpui::TestAppContext) {
    use crate::browser::{CoreGroupKey, Store};
    let group = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observation::set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"One"}]}),
            version: 3,
        });
        let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        let first = Store::update(cx, |store| store.open_core(&key, None)).unwrap();
        view.update(cx, |view, cx| {
            view.select_core_workspace(group, cx);
            view.show_browser_tab(first, window, cx);
            view.close_browser_tab(first, window, cx);
            assert!(view.browser_tab_ids(cx).is_empty());
            view.reopen_closed_tab(window, cx);
            let reopened = view.browser_tab_ids(cx);
            assert_eq!(reopened.len(), 1);
            assert_ne!(reopened[0], first);
            assert_eq!(cx.global::<Store>().get(reopened[0]).unwrap().core_group(), Some(key));
        });
    });
}

mod endpoint_independent_join;
mod hub_target;
#[cfg(unix)]
mod live;
mod remote_selection;

#[gpui::test]
fn selecting_a_group_shows_exact_member_identity_without_changing_the_document(
    cx: &mut gpui::TestAppContext,
) {
    let id = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [{
                    "id": id, "label": "One", "members": [
                        {"core_id": "core-a", "terminal_id": "t7"},
                        {"core_id": "core-b", "terminal_id": "t7"}
                    ]
                }]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.select_core_workspace(id, cx);
            assert_eq!(
                view.core_terminals
                    .workspace_ui
                    .selected_group
                    .as_ref()
                    .map(|key| key.group_id()),
                Some(id)
            );
            assert!(
                matches!(view.core_workspace_access(cx), Access::Ready(doc) if doc.version == 3)
            );
        });
    });
    cx.update(|window, cx| sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("core-member-core-a-t7").is_some());
    assert!(cx.debug_bounds("core-member-core-b-t7").is_some());
}

#[gpui::test]
fn selected_group_members_disappear_when_core_observation_is_stale(cx: &mut gpui::TestAppContext) {
    let id = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observation::set_document_for_test(cx, Doc {
            value: serde_json::json!({"schema_version": 1, "workspaces": [{
                "id": id, "label": "One", "members": [{"core_id": "core-a", "terminal_id": "t7"}]
            }]}),
            version: 3,
        });
        view.update(cx, |view, cx| view.select_core_workspace(id, cx));
    });
    cx.update(|window, cx| sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("core-member-core-a-t7").is_some());
    cx.update(|_, cx| observation::set_document_stale_for_test(cx));
    cx.update(|window, cx| sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("core-member-core-a-t7").is_none());
}

#[gpui::test]
fn a_narrow_selected_group_keeps_member_action_in_bounds(cx: &mut gpui::TestAppContext) {
    use gpui::{px, size};

    let id = Uuid::from_u128(71);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [{
                    "id": id, "label": "A long group name", "members": [{
                        "core_id": "core-a", "terminal_id": "very-long-terminal-identity"
                    }]
                }]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| view.select_core_workspace(id, cx));
    });
    cx.simulate_resize(size(px(320.), px(900.)));
    cx.update(|window, cx| sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let row = cx
        .debug_bounds("core-member-core-a-very-long-terminal-identity")
        .unwrap();
    let action = cx.debug_bounds("core-member-remove-00000000-0000-0000-0000-000000000047-core-a-very-long-terminal-identity").unwrap();
    assert!(action.left() >= row.left() && action.right() <= row.right());
    assert!(row.right() <= px(320.));
}

#[gpui::test]
fn group_actions_keep_their_full_uuid_and_delete_waits_for_confirmation(
    cx: &mut gpui::TestAppContext,
) {
    let first = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let second = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99b").unwrap();
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": first, "label": "Same"},
                    {"id": second, "label": "Same"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.open_core_workspace_rename(second, window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceRename));
            assert_eq!(
                view.core_terminals
                    .workspace_ui
                    .target
                    .map(|target| target.id),
                Some(second)
            );
            assert_eq!(
                view.menu.input.as_ref().map(|input| input.text.as_str()),
                Some("Same")
            );
            view.open_core_workspace_delete(first, window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceDelete));
            assert_eq!(
                view.core_terminals
                    .workspace_ui
                    .target
                    .map(|target| target.id),
                Some(first)
            );
            assert!(view.core_terminals.workspace_ui.pending.is_none());
        });
    });
}

#[gpui::test]
fn a_changed_document_refuses_an_old_delete_dialog(cx: &mut gpui::TestAppContext) {
    let id = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        for version in [3, 4] {
            observation::set_document_for_test(
                cx,
                Doc {
                    value: serde_json::json!({"schema_version": 1, "workspaces": [
                        {"id": id, "label": "One"}
                    ]}),
                    version,
                },
            );
            if version == 3 {
                view.update(cx, |view, cx| {
                    view.open_core_workspace_delete(id, window, cx)
                });
            }
        }
        view.update(cx, |view, cx| {
            view.submit_core_workspace_dialog(cx);
            assert!(view.core_terminals.workspace_ui.pending.is_none());
            assert!(view.menu.error.is_some());
        });
    });
}

#[gpui::test]
fn a_new_observer_epoch_refuses_the_old_target_at_the_same_version(cx: &mut gpui::TestAppContext) {
    let id = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        let doc = || Doc {
            value: serde_json::json!({"schema_version": 1, "workspaces": [
                {"id": id, "label": "One"}
            ]}),
            version: 3,
        };
        observation::set_document_for_test(cx, doc());
        view.update(cx, |view, cx| {
            view.open_core_workspace_delete(id, window, cx)
        });
        observation::set_document_for_test(cx, doc());
        view.update(cx, |view, cx| {
            view.submit_core_workspace_dialog(cx);
            assert!(view.core_terminals.workspace_ui.pending.is_none());
            assert!(view.menu.error.is_some());
        });
    });
}

#[gpui::test]
fn completed_create_does_not_close_another_core_dialog(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            view.core_terminals.workspace_ui.pending = Some(super::Pending {
                dialog_generation: view.core_terminals.workspace_ui.dialog_generation,
                dismiss_page: Some(Page::CoreWorkspaceCreate),
                receiver,
            });
            view.menu.page = Some(Page::CoreWorkspaceDelete);
            sender.send(Ok(())).unwrap();
            view.poll_core_workspace_edit(window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceDelete));
        });
    });
}

#[gpui::test]
fn completed_create_does_not_close_a_reopened_create_dialog(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            view.core_terminals.workspace_ui.pending = Some(super::Pending {
                dialog_generation: view.core_terminals.workspace_ui.dialog_generation,
                dismiss_page: Some(Page::CoreWorkspaceCreate),
                receiver,
            });
            view.open_core_workspace_dialog(window, cx);
            view.menu.input.as_mut().unwrap().text = "Second".into();
            sender.send(Ok(())).unwrap();
            view.poll_core_workspace_edit(window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceCreate));
            assert_eq!(
                view.menu.input.as_ref().map(|input| input.text.as_str()),
                Some("Second")
            );
        });
    });
}

#[gpui::test]
fn a_narrow_core_group_row_keeps_its_action_menu_in_bounds(cx: &mut gpui::TestAppContext) {
    use gpui::{px, size};

    let id = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let (_view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": id, "label": "A long Core workspace group name"}
                ]}),
                version: 3,
            },
        );
    });
    cx.simulate_resize(size(px(320.), px(900.)));
    cx.update(|window, cx| sidebar::layout_tests::full_draw(window, cx).clear(cx));
    let row = cx
        .debug_bounds("core-workspace-9ba7ac86-25e5-472a-b8b0-2c550760c99a")
        .unwrap();
    let menu = cx
        .debug_bounds("core-workspace-actions-9ba7ac86-25e5-472a-b8b0-2c550760c99a")
        .unwrap();
    assert!(menu.left() >= row.left() && menu.right() <= row.right());
    assert!(row.right() <= px(320.));
}

#[gpui::test]
fn core_group_menu_keyboard_reaches_delete_confirmation(cx: &mut gpui::TestAppContext) {
    use gpui::{point, px};

    let id = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        sidebar::layout_tests::fixture_core_window(window, cx)
    });
    cx.update(|window, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": id, "label": "One"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.open_core_workspace_actions(id, point(px(20.), px(100.)), window, cx)
        });
    });
    cx.simulate_keystrokes("down down enter");
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::CoreWorkspaceDelete))
    });
}

#[gpui::test]
fn archive_without_a_live_core_keeps_its_menu_open(cx: &mut gpui::TestAppContext) {
    use gpui::{point, px};

    let id = Uuid::parse_str("9ba7ac86-25e5-472a-b8b0-2c550760c99a").unwrap();
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observation::set_document_for_test(
            cx,
            Doc {
                value: serde_json::json!({"schema_version": 1, "workspaces": [
                    {"id": id, "label": "One"}
                ]}),
                version: 3,
            },
        );
        view.update(cx, |view, cx| {
            view.open_core_workspace_actions(id, point(px(20.), px(100.)), window, cx);
            view.cycle_core_workspace_action(true, cx);
            view.activate_selected_core_workspace_action(window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceActions));
            assert!(view.core_terminals.workspace_ui.pending.is_none());
        });
    });
}

#[test]
fn the_reader_never_treats_a_stale_document_as_empty() {
    let mut projection = observation::Projection::default();
    projection.begin(1);
    assert!(matches!(Access::of(Some(&projection)), Access::Missing));
    projection.initial(
        1,
        vec![],
        Ok(VersionedDocument::read(Doc {
            value: serde_json::Value::Null,
            version: 0,
        })
        .unwrap()),
    );
    assert!(matches!(Access::of(Some(&projection)), Access::Ready(_)));
    projection.lost(1);
    assert!(matches!(Access::of(Some(&projection)), Access::Missing));
}

#[gpui::test]
fn core_create_dialog_keeps_a_name_and_refuses_stale_submission(cx: &mut gpui::TestAppContext) {
    assert!(matches!(Access::of(None), Access::Missing));
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            assert_eq!(view.menu.page, Some(Page::CoreWorkspaceCreate));
            assert!(view.menu.input.is_some());
            view.menu.input.as_mut().unwrap().text = "One".into();
            view.submit_core_workspace_dialog(cx);
            assert!(view.core_terminals.workspace_ui.pending.is_none());
            assert!(matches!(view.core_workspace_access(cx), Access::Missing));
        });
    });
}

#[gpui::test]
fn uncertain_write_blocks_the_shared_version_until_core_observes_a_new_one(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            view.core_terminals.workspace_ui.pending = Some(super::Pending {
                dialog_generation: view.core_terminals.workspace_ui.dialog_generation,
                dismiss_page: Some(Page::CoreWorkspaceCreate),
                receiver,
            });
            let shared = EditGuard::begin(cx, 3).unwrap();
            let reason = Error::Timeout(Method::DocSet).to_string();
            let outcome = Err(super::edit::Error::WriteUnknown {
                version: 3,
                source: Error::Timeout(Method::DocSet),
            });
            EditGuard::finish(&shared, 3, &outcome);
            sender.send(outcome).unwrap();
            view.poll_core_workspace_edit(window, cx);
            assert!(
                view.menu
                    .error
                    .as_ref()
                    .is_some_and(|message| message.contains(&reason))
            );
        });
        assert!(EditGuard::blocks(cx, 3));
        let mut projection = observation::Projection::default();
        projection.begin(1);
        projection.initial(
            1,
            vec![],
            Ok(VersionedDocument::read(Doc {
                value: serde_json::Value::Null,
                version: 3,
            })
            .unwrap()),
        );
        let Access::Ready(document) = Access::of(Some(&projection)) else {
            panic!("Core document must be fresh");
        };
        assert!(EditGuard::blocks(cx, document.version));
        projection.initial(
            1,
            vec![],
            Ok(VersionedDocument::read(Doc {
                value: serde_json::Value::Null,
                version: 4,
            })
            .unwrap()),
        );
        let Access::Ready(document) = Access::of(Some(&projection)) else {
            panic!("new Core document must be fresh");
        };
        assert!(!EditGuard::blocks(cx, document.version));
    });
}

#[test]
fn an_uncertain_worker_keeps_the_version_fenced_after_its_window_closes() {
    let shared = std::sync::Arc::new(std::sync::Mutex::new(GuardState::Pending(3)));
    assert!(EditGuard::blocked(GuardState::Pending(3), 8));
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    drop(receiver);
    let worker_guard = shared.clone();
    std::thread::spawn(move || {
        let outcome = Err(super::edit::Error::WriteUnknown {
            version: 7,
            source: Error::Timeout(Method::DocSet),
        });
        EditGuard::finish(&worker_guard, 3, &outcome);
        let _ = sender.send(outcome);
    })
    .join()
    .unwrap();
    assert_eq!(*shared.lock().unwrap(), GuardState::Uncertain(7));
    assert!(EditGuard::blocked(GuardState::Uncertain(7), 4));
    assert!(EditGuard::blocked(GuardState::Uncertain(7), 7));
    assert!(!EditGuard::blocked(GuardState::Uncertain(7), 8));
}

#[gpui::test]
fn a_pending_core_pane_cannot_take_focus_after_its_group_is_hidden(cx: &mut gpui::TestAppContext) {
    use crate::{
        browser::{CoreGroupKey, Location, Store},
        core_terminals::{CoreTerminalView, Error, panes::Panes, worker::Link},
        sidebar,
    };
    use gpui::prelude::*;
    use sheprd_backend::TerminalId;
    use sheprd_core_client::wire::Doc;
    use uuid::Uuid;

    let first = Uuid::from_u128(71);
    let second = Uuid::from_u128(72);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observation::set_document_for_test_as(
            cx,
            "hub-a",
            Doc {
                value: serde_json::json!({"schema_version":1,"workspaces":[
                    {"id":first,"label":"One"},{"id":second,"label":"Two"}
                ]}),
                version: 3,
            },
        );
        let key = CoreGroupKey::new("hub-a", None, "spaceport", first).unwrap();
        let terminal = TerminalId::try_from("t7".to_owned()).unwrap();
        let panes = Panes::single(terminal.clone());
        let tab = Store::update(cx, |store| {
            store.open_core(
                &key,
                Some(Location::CoreTerminal {
                    panes: panes.clone(),
                    observe: false,
                }),
            )
        })
        .unwrap();
        view.update(cx, |view, cx| {
            view.core_terminals
                .hub
                .set_join_for_test(|_| Err(Error::NotInstalled));
            view.select_core_workspace(first, cx);
            view.show_browser_tab(tab, window, cx);
            let pane = cx.new(|cx| {
                CoreTerminalView::new(
                    terminal.clone(),
                    Link::for_test(),
                    view.core_look(),
                    view.painter.clone(),
                    false,
                    cx,
                )
            });
            view.core_terminals
                .show_for_test(tab, panes, false, Vec::new());
            view.core_terminals
                .views
                .insert((tab, terminal), pane.clone());
            view.core_terminals.focus_wanted = Some(tab);
            window.focus(&view.focus, cx);
            view.select_core_workspace(second, cx);
            view.poll_core_terminals(window, cx);
            assert!(!pane.read(cx).focus_handle().is_focused(window));
            assert_eq!(view.core_terminals.focus_wanted, None);
        });
    });
}
