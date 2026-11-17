// ABOUTME: Checks that catalog edits retire diagnostics from the previous target.
// ABOUTME: Drives catalog reconciliation through endpoint worker updates.
use super::*;

#[gpui::test]
fn catalog_retarget_forgets_the_previous_targets_failure(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, cx| {
        for failure in ["outage", "version", "missing"] {
            let saved = host("box", true);
            view.reconcile_catalog(vec![saved.clone()], Vec::new(), cx);
            let endpoint = &mut view.endpoints[1];
            let old_inbox = endpoint.connection.inbox.clone();
            {
                let mut state = old_inbox.lock().unwrap();
                if failure == "version" {
                    state.apply(ClientEvent::VersionMismatch(
                        VersionMismatch::DaemonOutdated {
                            server_version: None,
                        },
                    ));
                }
                state.missing_installation = failure == "missing";
                state.apply(ClientEvent::Disconnected {
                    reason: "old target failed".into(),
                    ssh: None,
                });
            }
            endpoint.poll(Instant::now());
            assert_ne!(endpoint.phase(), Phase::Connecting, "{failure}");

            view.reconcile_catalog(
                vec![SavedHost {
                    target: "replacement".into(),
                    ..saved
                }],
                Vec::new(),
                cx,
            );
            let endpoint = &view.endpoints[1];
            assert_eq!(endpoint.phase(), Phase::Connecting, "{failure}");
            assert_eq!(endpoint.outage(), None, "{failure}");
            assert!(!endpoint.herdr_missing(), "{failure}");
            assert!(endpoint.live.notices.is_none(), "{failure}");
            assert!(!Arc::ptr_eq(&old_inbox, &endpoint.connection.inbox));
        }
    });
}

#[test]
fn stopping_an_attempt_keeps_it_eligible_for_a_retry() {
    let mut endpoint = Endpoint::new(LOCAL.into(), "Local".into(), ConnectTarget::Local, true);
    endpoint.stop();
    assert!(endpoint.may_dial());
    assert_eq!(endpoint.phase(), Phase::Connecting);
    assert_eq!(endpoint.live.status, ConnectionStatus::Connecting);
}

#[test]
fn stopping_a_worker_diagnostic_without_an_outage_starts_fresh() {
    let mut endpoint = Endpoint::new(LOCAL.into(), "Local".into(), ConnectTarget::Local, true);
    endpoint
        .connection
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Disconnected {
            reason: "attempt failed".into(),
            ssh: None,
        });
    endpoint.poll(Instant::now());
    assert_eq!(endpoint.phase(), Phase::Reconnecting);
    assert_eq!(endpoint.outage(), None);
    endpoint.stop();
    assert_eq!(endpoint.phase(), Phase::Connecting);
}
