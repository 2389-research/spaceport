// ABOUTME: Checks tab commands against caller workspace and daemon boundaries.
// ABOUTME: Refuses hidden and unshown tab targets without changing the shared Store.
use super::*;

#[gpui::test]
fn caller_workspace_filters_lists_and_refuses_other_tabs(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let own = open(&view, cx, "w0", None);
    let other = open(&view, cx, "w1", None);
    let caller = Caller {
        workspace_id: Some("w0".into()),
        ..Caller::default()
    };
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let (call, incoming, waiting) = queued(&server, TabOp::List {}, caller.clone());
    cx.cx
        .update(|cx| serve(call, incoming, &mut Watches::default(), cx));
    let Response::Tab {
        reply: Reply::Tabs { tabs },
    } = waiting.join().unwrap().unwrap()
    else {
        panic!("not tabs")
    };
    assert_eq!(
        tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
        [u64::from(own)]
    );
    cx.cx.update(|cx| {
        for response in [
            focus(&caller, other.into(), cx),
            close(&caller, other.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        assert!(cx.global::<Store>().get(other).is_some());
    });
}

#[gpui::test]
fn an_unshown_session_cannot_focus_or_close_another_sessions_tab(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let tab = open(&view, cx, "w0", None);
    let caller = Caller {
        workspace_id: Some("w0".into()),
        daemon_socket: Some("/tmp/unshown-session.sock".into()),
        ..Caller::default()
    };
    cx.cx.update(|cx| {
        for response in [
            focus(&caller, tab.into(), cx),
            close(&caller, tab.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::WorkspaceNotFound | ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        assert!(cx.global::<Store>().get(tab).is_some());
    });
}

#[gpui::test]
fn hidden_tabs_cannot_be_listed_focused_or_closed_in_the_selected_window(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = window(cx);
    let tab = open(&view, cx, "w0", None);
    cx.cx.update(|cx| {
        view.update(cx, |view, _| {
            let key = (crate::browser::scope(&view.endpoints[0]), "w0".into());
            view.browser
                .hidden
                .hide(key, [crate::browser::Pick::Page(tab)]);
        });
        for response in [
            focus(&Caller::default(), tab.into(), cx),
            close(&Caller::default(), tab.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        assert!(cx.global::<Store>().get(tab).is_some());
    });
}

#[gpui::test]
fn a_workspace_without_a_daemon_stays_on_the_resolved_endpoint(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    let own = open(&view, cx, "w0", None);
    let other = cx.cx.update(|cx| {
        let target = sheprd_client::ConnectTarget::Socket("/tmp/other-control-session.sock".into());
        view.update(cx, |view, _| {
            let mut endpoint =
                crate::endpoint::Endpoint::new("other".into(), "Other".into(), target, true);
            endpoint.live.snapshot = Some(Arc::new(snapshot(2)));
            view.endpoints.push(endpoint);
        });
        Store::update(cx, |store| {
            store.open(
                Scope::local(Path::new("/tmp/other-control-session.sock")),
                "w0",
                Some(Location::Devices),
                None,
            )
        })
        .unwrap()
    });
    let caller = Caller {
        workspace_id: Some("w0".into()),
        ..Caller::default()
    };
    cx.cx.update(|cx| {
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list_for(&caller, cx)
        else {
            panic!("not tabs")
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(own)]
        );
        for response in [
            focus(&caller, other.into(), cx),
            close(&caller, other.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        assert!(cx.global::<Store>().get(other).is_some());
    });
}

#[gpui::test]
fn ambiguous_sessions_require_a_window_and_named_windows_never_search_elsewhere(
    cx: &mut gpui::TestAppContext,
) {
    let first = cx.add_window(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(2)));
        view
    });
    let second = cx.add_window(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.endpoints[0] = crate::endpoint::Endpoint::new(
            "other".into(),
            "Other".into(),
            sheprd_client::ConnectTarget::Socket("/tmp/other-session-window.sock".into()),
            true,
        );
        view.live.snapshot = Some(Arc::new(snapshot(2)));
        view
    });
    cx.update(|cx| {
        let own_scope = first
            .read_with(cx, |view, _| crate::browser::scope(&view.endpoints[0]))
            .unwrap();
        let other_scope = second
            .read_with(cx, |view, _| crate::browser::scope(&view.endpoints[0]))
            .unwrap();
        let (own, other) = Store::update(cx, |store| {
            (
                store
                    .open(own_scope, "w0", Some(Location::Devices), None)
                    .unwrap(),
                store
                    .open(other_scope, "w0", Some(Location::Devices), None)
                    .unwrap(),
            )
        });
        let caller = Caller {
            workspace_id: Some("w0".into()),
            ..Caller::default()
        };
        assert!(matches!(
            list_for(&caller, cx),
            Response::Error {
                code: ErrorCode::NotFound,
                ..
            }
        ));
        for response in [
            focus(&caller, own.into(), cx),
            close(&caller, own.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        let named = Caller {
            window: Some(first.window_id().as_u64()),
            ..caller
        };
        let Response::Tab {
            reply: Reply::Tabs { tabs },
        } = list_for(&named, cx)
        else {
            panic!("not tabs")
        };
        assert_eq!(
            tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [u64::from(own)]
        );
        for response in [
            focus(&named, other.into(), cx),
            close(&named, other.into(), cx),
        ] {
            assert!(
                matches!(
                    response,
                    Response::Error {
                        code: ErrorCode::NotFound,
                        ..
                    }
                ),
                "{response:?}"
            );
        }
        assert!(cx.global::<Store>().get(own).is_some());
        assert!(cx.global::<Store>().get(other).is_some());
    });
}
