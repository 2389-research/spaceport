// ABOUTME: Verifies partial policy updates and isolation from parallel readers.
// ABOUTME: Exercises the same cached values the application and themes consume.
use super::*;
use test_support::Scope;

#[test]
fn missing_preferences_keep_the_last_snapshot_and_do_not_leak_to_other_threads() {
    let _scope = Scope::new();
    store(Preferences {
        high_contrast: Some(true),
        reduce_motion: Some(true),
    });
    store(Preferences::default());
    assert!(reduce_motion() && high_contrast());
    assert_eq!(
        std::thread::spawn(snapshot).join().ok(),
        Some(POLICY.load(Ordering::Relaxed))
    );
    store(Preferences {
        high_contrast: None,
        reduce_motion: Some(false),
    });
    assert!(!reduce_motion());
    assert!(high_contrast());
}
