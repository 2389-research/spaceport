// ABOUTME: Checks native tab targeting against exact named windows and workspaces.
// ABOUTME: Opts into Core only when the scenario targets a Core group.
//! Tab requests against a headless window showing workspace `w0`.
use super::*;

#[gpui::test]
fn core_tab_focus_requires_the_exact_observed_group_and_named_window(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(|cx| crate::core_feature::install(true, cx));
    use crate::{browser::CoreGroupKey, core_terminals::set_document_for_test_as};
    use sheprd_core_client::wire::Doc;
    let group = uuid::Uuid::from_u128(71);
    let other = uuid::Uuid::from_u128(72);
    let (view, cx) = window(cx);
    let tab = cx.cx.update(|cx| {
        set_document_for_test_as(cx, "hub-a", Doc {
            value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Same"},{"id":other,"label":"Same"}]}),
            version: 3,
        });
        let key = CoreGroupKey::new("hub-a", None, "spaceport", group).unwrap();
        Store::update(cx, |store| store.open_core(&key, None)).unwrap()
    });
    cx.cx.update(|cx| {
        assert!(matches!(
            focus(
                &Caller {
                    window: Some(999_999),
                    ..Caller::default()
                },
                tab.into(),
                cx
            ),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        assert!(matches!(
            focus(&Caller::default(), tab.into(), cx),
            Response::Done { .. }
        ));
        let selected = view.read(cx).active_browser_key(cx).unwrap();
        assert_eq!(
            selected,
            CoreGroupKey::new("hub-a", None, "spaceport", group)
                .unwrap()
                .parts()
        );
        let background = CoreGroupKey::new("hub-a", None, "spaceport", other).unwrap();
        let background_tab = Store::update(cx, |store| store.open_core(&background, None)).unwrap();
        assert!(matches!(
            close(&Caller::default(), background_tab.into(), cx),
            Response::Done { .. }
        ));
        assert_eq!(view.read(cx).active_browser_key(cx).unwrap(), selected);
    });
    cx.cx.update(|cx| set_document_for_test_as(cx, "hub-b", Doc {
        value: serde_json::json!({"schema_version":1,"workspaces":[{"id":group,"label":"Same"}]}), version: 4,
    }));
    cx.cx.update(|cx| {
        assert!(matches!(
            focus(&Caller::default(), tab.into(), cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
    });
    cx.update(|window, cx| {
        let saved = cx.global::<Store>().get(tab).cloned().unwrap();
        assert!(!view.update(cx, |view, cx| close_in_window(view, &saved, window, cx)));
        assert!(cx.global::<Store>().get(tab).is_some());
    });
}

#[gpui::test]
fn tabs_are_listed_focused_and_closed(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let page = open(&view, cx, "w0", Some(Location::Devices));
    let devices = open(&view, cx, "w1", Some(Location::Devices));
    let caller = Caller::default();
    cx.cx.update(|cx| {
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list(cx)
        else {
            panic!("not a tab list");
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(page), u64::from(devices)]
        );
        // A tab of another workspace the window lists is shown there.
        assert!(matches!(
            focus(&caller, devices.into(), cx),
            Response::Done { .. }
        ));
        assert!(matches!(
            focus(&caller, 99, cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        assert!(matches!(
            close(&caller, page.into(), cx),
            Response::Done { .. }
        ));
        assert!(cx.global::<Store>().get(page).is_none());
        assert!(matches!(
            close(&caller, page.into(), cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
    });
}

#[gpui::test]
fn a_tab_of_a_workspace_no_window_lists_is_not_found(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let elsewhere = open(&view, cx, "w_gone", None);
    cx.cx.update(|cx| {
        assert!(matches!(
            focus(&Caller::default(), elsewhere.into(), cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        // A named window that does not show the tab closes nothing.
        let named = Caller {
            window: Some(999_999),
            ..Caller::default()
        };
        assert!(matches!(
            close(&named, elsewhere.into(), cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        assert!(cx.global::<Store>().get(elsewhere).is_some());
        // Closing needs no window: the store forgets it, and windows follow.
        assert!(matches!(
            close(&Caller::default(), elsewhere.into(), cx),
            Response::Done { .. }
        ));
        assert!(cx.global::<Store>().get(elsewhere).is_none());
    });
}

#[gpui::test]
fn devices_open_in_the_shown_workspace_at_once(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx);
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let (call, incoming, waiting) = queued(&server, TabOp::Devices, Caller::default());
    let mut watches = Watches::default();
    cx.cx.update(|cx| serve(call, incoming, &mut watches, cx));
    let Response::Tab {
        reply: Reply::Opened { id, workspace },
    } = waiting.join().unwrap().unwrap()
    else {
        panic!("not opened");
    };
    assert_eq!(workspace, "w0");
    cx.cx.update(|cx| {
        let tab = cx.global::<Store>().get(TabId::from(id)).cloned().unwrap();
        assert_eq!(tab.location, Some(Location::Devices));
    });
}

#[gpui::test]
fn a_tab_for_another_workspace_waits_for_the_switch(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let caller = Caller {
        workspace_id: Some("w1".into()),
        ..Caller::default()
    };
    let (call, incoming, waiting) = queued(&server, TabOp::Devices, caller);
    let mut watches = Watches::default();
    cx.cx.update(|cx| serve(call, incoming, &mut watches, cx));
    // Nothing opens while the window still shows w0.
    cx.cx.update(|cx| {
        watches.serve(cx);
        assert_eq!(watches.0.len(), 1);
        assert!(
            cx.try_global::<Store>()
                .is_none_or(|store| store.tabs().count() == 0)
        );
    });
    // The daemon's next snapshot says the switch landed.
    cx.cx.update(|cx| {
        view.update(cx, |view, _| {
            let mut shown = snapshot(2);
            shown.focused_workspace_id = Some("w1".into());
            view.live.snapshot = Some(Arc::new(shown));
        });
        watches.serve(cx);
        assert!(watches.0.is_empty());
    });
    assert!(matches!(
        waiting.join().unwrap().unwrap(),
        Response::Tab {
            reply: Reply::Opened { workspace, .. },
        } if workspace == "w1"
    ));
}

#[gpui::test]
fn requests_for_unknown_workspaces_are_refused(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx);
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let mut watches = Watches::default();
    let missing = Caller {
        workspace_id: Some("w_missing".into()),
        ..Caller::default()
    };
    let (call, incoming, waiting) = queued(&server, TabOp::Devices, missing);
    cx.cx.update(|cx| serve(call, incoming, &mut watches, cx));
    assert!(matches!(
        waiting.join().unwrap().unwrap(),
        Response::Error {
            code: ErrorCode::WorkspaceNotFound,
            ..
        }
    ));
    assert!(watches.0.is_empty());
}
