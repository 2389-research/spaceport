// ABOUTME: Checks catalog retargeting after a real isolated transport outage.
// ABOUTME: Proves late reports from the retired connection stay fenced out.
use super::*;

#[gpui::test]
fn changing_a_saved_host_clears_a_real_transports_outage(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let (endpoint, server) = connected_endpoint("ssh:box");
    view.update(cx, |view, cx| {
        view.reconcile_catalog(vec![tests::host("box", true)], Vec::new(), cx);
        view.endpoints[1] = endpoint;
        server.stream.shutdown(std::net::Shutdown::Both).unwrap();
        wait_until(|| {
            view.endpoints[1].poll(Instant::now());
            view.endpoints[1].connection.handle.is_none()
        });
        assert!(view.endpoints[1].outage().is_some());
        let old_inbox = view.endpoints[1].connection.inbox.clone();
        let mut saved = tests::host("box", true);
        saved.target = "replacement".into();
        view.reconcile_catalog(vec![saved], Vec::new(), cx);
        let endpoint = &mut view.endpoints[1];
        assert_eq!(endpoint.phase(), Phase::Connecting);
        assert_eq!(endpoint.outage(), None);
        old_inbox.lock().unwrap().apply(
            ClientEvent::Snapshot(Arc::new(snapshot())),
            &crate::state::labels(),
        );
        endpoint.poll(Instant::now());
        assert_eq!(endpoint.phase(), Phase::Connecting);
        assert!(endpoint.live().snapshot.is_none());
    });
}
