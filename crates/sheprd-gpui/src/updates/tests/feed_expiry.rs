// ABOUTME: Prevents Sparkle's timed recovery from accepting an invalid signed feed.
// ABOUTME: Requires explicit numeric zero, rejecting absent and nonzero intervals.
use super::*;

#[test]
fn invalid_signed_feed_must_never_gain_eligibility_after_time_passes() {
    let mut bundle = preview();
    assert!(accepts(&bundle));
    for interval in [None, Some(1_728_000.0), Some(-1.0), Some(f64::NAN)] {
        bundle.signed_feed_failure_expiration = interval;
        assert!(
            !accepts(&bundle),
            "accepted expiration interval {interval:?}"
        );
    }
}
