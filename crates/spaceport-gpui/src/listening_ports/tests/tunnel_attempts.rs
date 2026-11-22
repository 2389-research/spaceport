// ABOUTME: Tests tunnel attempt fencing and lifecycle ownership.
// ABOUTME: Uses exact owned children to verify cancellation and cleanup.

use super::*;

fn key(port: u16) -> Key {
    Key {
        target: "devbox.invalid".into(),
        address: None,
        port,
    }
}

fn start(tunnels: &mut tunnel::Tunnels, key: &Key) -> tunnel::Attempt {
    assert!(tunnels.begin(key).is_some());
    tunnels.attempt(key).unwrap()
}

/// Every path that cancels an attempt before a second click starts one.
const CANCELLATIONS: &[fn(&mut tunnel::Tunnels, &Key)] = &[
    |tunnels, key| {
        tunnels.close(key);
    },
    |tunnels, _| {
        tunnels.retain(|_| false);
    },
    |tunnels, _| tunnels.close_all(),
];

#[test]
fn a_stale_failure_leaves_the_replacement_attempt_wanted() {
    for cancel in CANCELLATIONS {
        let mut tunnels = tunnel::Tunnels::default();
        let key = key(5173);
        let old = start(&mut tunnels, &key);
        cancel(&mut tunnels, &key);
        let current = start(&mut tunnels, &key);
        assert!(
            !tunnels.fail(&key, &old),
            "stale failure consumed attempt B"
        );
        current.adopt(idle_child());
        assert_eq!(
            tunnels.finish(key.clone(), &current, tunnel::Tunnel::new(&current, 41010)),
            Some(41010)
        );
        assert_eq!(tunnels.local(&key), Some(41010));
        assert!(!tunnels.fail(&key, &old));
        assert_eq!(tunnels.local(&key), Some(41010));
    }
}

#[test]
fn a_stale_success_is_reaped_and_leaves_the_replacement_wanted() {
    for cancel in CANCELLATIONS {
        for complete_current_first in [false, true] {
            let mut tunnels = tunnel::Tunnels::default();
            let key = key(5173);
            let old = start(&mut tunnels, &key);
            let child = idle_child();
            let old_pid = child.id();
            old.adopt(child);
            let late = tunnel::Tunnel::new(&old, 41010);
            cancel(&mut tunnels, &key);
            let current = start(&mut tunnels, &key);
            if complete_current_first {
                current.adopt(idle_child());
                assert_eq!(
                    tunnels.finish(key.clone(), &current, tunnel::Tunnel::new(&current, 41011)),
                    Some(41011)
                );
            }
            assert_eq!(
                tunnels.finish(key.clone(), &old, late),
                None,
                "stale success consumed attempt B"
            );
            assert!(gone(old_pid));
            if !complete_current_first {
                current.adopt(idle_child());
                assert_eq!(
                    tunnels.finish(key.clone(), &current, tunnel::Tunnel::new(&current, 41011)),
                    Some(41011)
                );
            }
            assert_eq!(tunnels.local(&key), Some(41011));
        }
    }
}

#[test]
fn a_retry_keeps_the_open_tunnel_until_its_replacement_succeeds() {
    let mut tunnels = tunnel::Tunnels::default();
    let key = key(5173);
    let first = start(&mut tunnels, &key);
    let child = idle_child();
    let first_pid = child.id();
    first.adopt(child);
    assert_eq!(
        tunnels.finish(key.clone(), &first, tunnel::Tunnel::new(&first, 41010)),
        Some(41010)
    );
    let failed = start(&mut tunnels, &key);
    assert_eq!(tunnels.local(&key), Some(41010));
    assert!(tunnels.fail(&key, &failed));
    assert_eq!(tunnels.local(&key), Some(41010));
    let replacement = start(&mut tunnels, &key);
    replacement.adopt(idle_child());
    assert_eq!(
        tunnels.finish(
            key.clone(),
            &replacement,
            tunnel::Tunnel::new(&replacement, 41011)
        ),
        Some(41011)
    );
    assert!(gone(first_pid));
    assert_eq!(tunnels.local(&key), Some(41011));
}

#[test]
fn opening_attempts_are_bounded_and_release_capacity_after_cancellation() {
    let mut tunnels = tunnel::Tunnels::default();
    for port in 10000..10032 {
        start(&mut tunnels, &key(port));
    }
    assert_eq!(tunnels.begin(&key(11000)), None);
    assert!(tunnels.close(&key(10000)));
    assert!(tunnels.begin(&key(11000)).is_some());
}
