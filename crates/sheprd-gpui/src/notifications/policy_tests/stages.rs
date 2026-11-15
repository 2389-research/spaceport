// ABOUTME: Pins the stages one notice moves through: waiting, ready, visible or posted.
// ABOUTME: Covers pausing, eviction from a full queue, and the QA preview that shows at once.
use super::*;

/// Every retained notice's stage, host by host, in queue order.
fn stages(endpoints: &[Endpoint]) -> Vec<&'static str> {
    endpoints
        .iter()
        .flat_map(|e| e.toasts.entries.iter())
        .map(|(_, n)| stage(n))
        .collect()
}

/// No retained notice may read as both a toast and an OS notification.
fn assert_one_surface(endpoints: &[Endpoint]) {
    assert!(
        endpoints
            .iter()
            .flat_map(|e| e.toasts.entries.iter())
            .all(|(_, n)| !visible_and_posted(n))
    );
}

fn confirm_blocked(endpoints: &mut [Endpoint]) {
    Arc::make_mut(endpoints[1].live.snapshot.as_mut().unwrap()).agents[0].agent_status =
        AgentStatus::Blocked;
}

/// A daemon notice with no pane, so none replaces another.
fn targetless(title: &str, now: Instant) -> Notice {
    let mut event = tests::notification(title);
    event.kind = Kind::Custom;
    Notice::new(event, now)
}

#[test]
fn a_toast_waits_becomes_ready_shows_and_leaves_on_expiry() {
    let now = Instant::now();
    let mut endpoints = endpoints();
    receive(&mut endpoints[1], wire(Kind::NeedsAttention), now);
    assert_eq!(stages(&endpoints), ["waiting"]);
    // Waiting out its delay, then for the agent's state to confirm it.
    tick(&mut endpoints, 0, config(1), false, now);
    assert_eq!(stages(&endpoints), ["waiting"]);
    let later = now + Duration::from_secs(1);
    confirm_blocked(&mut endpoints);
    // Ready, but not shown while toasts are hidden.
    tick(&mut endpoints, 0, config(1), true, later);
    assert_eq!(stages(&endpoints), ["ready"]);
    assert_eq!(expires(&endpoints[1].toasts.entries[0].1), None);
    let shown = later + Duration::from_secs(3);
    tick(&mut endpoints, 0, config(1), false, shown);
    assert_eq!(stages(&endpoints), ["visible"]);
    let leaves = shown + Duration::from_secs(8);
    assert_eq!(expires(&endpoints[1].toasts.entries[0].1), Some(leaves));
    tick(
        &mut endpoints,
        0,
        config(1),
        false,
        leaves - Duration::from_nanos(1),
    );
    assert_eq!(stages(&endpoints), ["visible"]);
    assert!(tick(&mut endpoints, 0, config(1), false, leaves));
    assert!(stages(&endpoints).is_empty());
}

#[test]
fn an_os_notification_waits_becomes_ready_and_is_posted() {
    let now = Instant::now();
    let mut endpoints = endpoints();
    receive(&mut endpoints[1], wire(Kind::NeedsAttention), now);
    tick(&mut endpoints, 0, system(1), false, now);
    assert_eq!(stages(&endpoints), ["waiting"]);
    assert!(take_system(&mut endpoints, system(1)).is_empty());
    assert_eq!(stages(&endpoints), ["waiting"]);
    let later = now + Duration::from_secs(1);
    confirm_blocked(&mut endpoints);
    tick(&mut endpoints, 0, system(1), false, later);
    // Ready notices wait for the poll loop; ticking never posts or shows.
    assert_eq!(stages(&endpoints), ["ready"]);
    tick(&mut endpoints, 0, system(1), false, later);
    assert_eq!(stages(&endpoints), ["ready"]);
    assert_eq!(take_system(&mut endpoints, system(1)).len(), 1);
    assert_eq!(stages(&endpoints), ["posted"]);
    // Posted has no lifetime: it stays long after any toast would have left.
    tick(
        &mut endpoints,
        0,
        system(1),
        false,
        later + Duration::from_secs(3600),
    );
    assert_eq!(stages(&endpoints), ["posted"]);
    assert_eq!(expires(&endpoints[1].toasts.entries[0].1), None);
}

#[test]
fn a_pending_navigation_pauses_only_its_own_toast() {
    let now = Instant::now();
    for (selected, pending, paused) in [
        (1, Some(0), true),
        // Another host's toast shares the id; another id is another toast.
        (0, Some(0), false),
        (1, Some(7), false),
        (1, None, false),
    ] {
        let mut endpoints = endpoints();
        endpoints[1]
            .toasts
            .receive([targetless("toast", now).preview()]);
        let full = |endpoints: &mut [Endpoint], pending, time| {
            super::super::tick(
                endpoints,
                selected,
                config(0),
                None,
                false,
                true,
                pending,
                time,
            )
        };
        full(&mut endpoints, None, now);
        let leaves = now + Duration::from_secs(5);
        assert_eq!(expires(&endpoints[1].toasts.entries[0].1), Some(leaves));
        let held = now + Duration::from_secs(100);
        full(&mut endpoints, pending, held);
        if !paused {
            assert!(stages(&endpoints).is_empty(), "{selected} {pending:?}");
            continue;
        }
        // Each tick that is paused, and the one that ends the pause, adds
        // the time since the tick before it to the toast's lifetime.
        let extended = |seconds| Some(leaves + Duration::from_secs(seconds));
        assert_eq!(stages(&endpoints), ["visible"]);
        assert_eq!(expires(&endpoints[1].toasts.entries[0].1), extended(100));
        let resumed = held + Duration::from_secs(100);
        full(&mut endpoints, pending, resumed);
        assert_eq!(expires(&endpoints[1].toasts.entries[0].1), extended(200));
        let after = resumed + Duration::from_secs(100);
        full(&mut endpoints, None, after);
        assert_eq!(stages(&endpoints), ["visible"]);
        assert_eq!(expires(&endpoints[1].toasts.entries[0].1), extended(300));
        // Running again, it is not extended further.
        full(&mut endpoints, None, after + Duration::from_secs(1));
        assert_eq!(expires(&endpoints[1].toasts.entries[0].1), extended(300));
        full(&mut endpoints, None, leaves + Duration::from_secs(300));
        assert!(stages(&endpoints).is_empty());
    }
}

#[test]
fn a_full_queue_evicts_its_first_entry_that_is_not_a_showing_toast() {
    let now = Instant::now();
    let full = PENDING_LIMIT * 2 + 1;
    let titles = |endpoint: &Endpoint| -> Vec<String> {
        endpoint
            .toasts
            .entries
            .iter()
            .map(|(_, n)| n.title.clone())
            .collect()
    };

    // A showing toast at the front is passed over for the entry behind it.
    let mut endpoints = endpoints();
    endpoints[1].toasts.receive([targetless("shown", now)]);
    tick(&mut endpoints, 0, config(0), false, now);
    assert_eq!(stages(&endpoints), ["visible"]);
    endpoints[1]
        .toasts
        .receive((1..full).map(|i| targetless(&i.to_string(), now)));
    assert_eq!(endpoints[1].toasts.entries.len(), full);
    endpoints[1].toasts.receive([targetless("new", now)]);
    assert_eq!(endpoints[1].toasts.entries.len(), full);
    let kept = titles(&endpoints[1]);
    assert_eq!(kept[0], "shown");
    assert_eq!(kept[1], "2");
    assert_eq!(kept[full - 1], "new");

    // A posted notice at the front goes, although it still answers its
    // click, while every waiting notice behind it stays.
    let mut endpoints = self::endpoints();
    endpoints[1].toasts.receive([targetless("posted", now)]);
    tick(&mut endpoints, 0, system(0), false, now);
    assert_eq!(take_system(&mut endpoints, system(0)).len(), 1);
    assert_eq!(stages(&endpoints), ["posted"]);
    endpoints[1]
        .toasts
        .receive((1..full).map(|i| targetless(&i.to_string(), now)));
    assert_eq!(endpoints[1].toasts.entries.len(), full);
    assert_eq!(posted(&endpoints), 1);
    endpoints[1].toasts.receive([targetless("new", now)]);
    assert_eq!(endpoints[1].toasts.entries.len(), full);
    assert_eq!(posted(&endpoints), 0);
    let kept = titles(&endpoints[1]);
    assert_eq!(kept[0], "1");
    assert_eq!(kept[full - 1], "new");
    assert!(stages(&endpoints).iter().all(|stage| *stage == "waiting"));
}

#[test]
fn a_preview_promoted_before_it_is_received_shows_at_once() {
    let now = Instant::now();
    let mut endpoints = endpoints();
    // Notifications are off; a preview shows anyway.
    let off = NotificationConfig::default();
    let mut notice = Notice::new(wire(Kind::NeedsAttention), now)
        .preview()
        .with_snapshot(endpoints[0].live.snapshot.as_deref());
    notice.promote(now);
    endpoints[0].toasts.receive([notice]);
    assert_eq!(stages(&endpoints), ["visible"]);
    let leaves = now + Duration::from_secs(8);
    assert_eq!(expires(&endpoints[0].toasts.entries[0].1), Some(leaves));
    // Its lifetime runs from the promotion, not from the first tick.
    tick(&mut endpoints, 0, off, false, now + Duration::from_secs(2));
    assert_eq!(stages(&endpoints), ["visible"]);
    assert_eq!(expires(&endpoints[0].toasts.entries[0].1), Some(leaves));
    assert!(take_system(&mut endpoints, system(0)).is_empty());
    assert_eq!(stages(&endpoints), ["visible"]);
    tick(&mut endpoints, 0, off, false, leaves);
    assert!(stages(&endpoints).is_empty());
}

#[test]
fn a_notice_is_never_both_a_toast_and_an_os_notification() {
    let now = Instant::now();
    // A showing toast is not posted when delivery moves to the OS center.
    let mut endpoints = endpoints();
    endpoints[1].toasts.receive([targetless("toast", now)]);
    tick(&mut endpoints, 0, config(0), false, now);
    assert_eq!(stages(&endpoints), ["visible"]);
    tick(&mut endpoints, 0, system(0), false, now);
    assert!(take_system(&mut endpoints, system(0)).is_empty());
    assert_eq!(stages(&endpoints), ["visible"]);
    assert_one_surface(&endpoints);

    // A posted notice is not shown when delivery moves back in-app, even
    // with nothing else to show.
    let mut endpoints = self::endpoints();
    endpoints[1].toasts.receive([targetless("post", now)]);
    tick(&mut endpoints, 0, system(0), false, now);
    assert_eq!(take_system(&mut endpoints, system(0)).len(), 1);
    for step in 0..3 {
        tick(
            &mut endpoints,
            0,
            config(0),
            false,
            now + Duration::from_secs(step),
        );
        assert_eq!(stages(&endpoints), ["posted"]);
        assert!(visible(&endpoints).is_empty());
        assert_one_surface(&endpoints);
    }
    // Nor is it posted a second time.
    tick(&mut endpoints, 0, system(0), false, now);
    assert!(take_system(&mut endpoints, system(0)).is_empty());
    assert_one_surface(&endpoints);
}
