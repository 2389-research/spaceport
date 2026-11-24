use super::*;
use crate::endpoint::WakeClock;
use std::time::Duration;

#[gpui::test]
fn waking_redials_dropped_endpoints_now_and_leaves_detached_ones(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, cx| {
        view.reconcile_catalog(
            vec![
                crate::endpoint::tests::host("down", true),
                crate::endpoint::tests::host("parked", true),
            ],
            Vec::new(),
            cx,
        );
        let now = Instant::now();
        let later = now + Duration::from_secs(30);
        for endpoint in &mut view.endpoints {
            endpoint.connection.handle = None;
            endpoint.set_retry_at_for_test(later);
        }
        view.endpoints[2].detach_for_test();
        let wall = SystemTime::now();
        view.wake = WakeClock::new(now, wall, NetworkChanges::default());

        let quiet = now + Duration::from_secs(1);
        assert!(!view.recover_connections(
            quiet,
            wall + Duration::from_secs(1),
            NetworkChanges::default()
        ));
        for endpoint in &mut view.endpoints {
            endpoint.poll(quiet);
        }
        assert!(view.endpoints.iter().all(|e| e.retry_at() == later));

        let woke = now + Duration::from_secs(2);
        assert!(view.recover_connections(
            woke,
            wall + Duration::from_secs(600),
            NetworkChanges::default()
        ));
        // Each endpoint recovers in its own poll, after taking its update.
        assert_eq!(view.endpoints[1].retry_at(), later);
        for endpoint in &mut view.endpoints {
            endpoint.poll(woke);
        }
        assert_eq!(view.endpoints[1].retry_at(), woke);
        // A detached endpoint is never dialled by the poll loop, whatever its time.
        assert!(view.endpoints[2].detached());
    });
}
