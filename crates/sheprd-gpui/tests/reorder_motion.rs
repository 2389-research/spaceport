// ABOUTME: Checks reorder timing from the normal dependency graph used by native test binaries.
// ABOUTME: Prevents headless test features from disabling real application slides.
use sheprd_reorder::{SLIDE, Slide};
use std::time::{Duration, Instant};

#[test]
fn native_test_consumers_keep_real_reorder_progression() {
    let start = Instant::now();
    let slide = Slide {
        from: 0.,
        to: 10.,
        start,
    };
    let progress = slide.progress(start + Duration::from_millis(75), SLIDE);
    assert!(
        progress > 0. && progress < 1.,
        "normal dependency consumers must retain intermediate motion: {progress}"
    );
    assert_eq!(SLIDE, Duration::from_millis(150));
}
