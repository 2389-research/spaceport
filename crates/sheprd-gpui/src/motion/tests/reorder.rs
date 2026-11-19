// ABOUTME: Applies reduced motion to real-duration reorder slides.
// ABOUTME: Keeps unit-test settling from hiding the native timing policy.
use super::*;
use sheprd_reorder::{SLIDE, Slide};

#[gpui::test]
fn reduced_motion_settles_a_real_duration_slide_and_stops_its_frames(
    cx: &mut gpui::TestAppContext,
) {
    let _scope = crate::accessibility::policy::tests::Scope::new();
    let start = Instant::now();
    let slide = Slide {
        from: 0.,
        to: 10.,
        start,
    };
    let middle = start + SLIDE / 2;
    let normal = slide_duration(SLIDE);
    assert_eq!(normal, SLIDE);
    assert!(slide.at(middle, normal) > 0. && slide.at(middle, normal) < 10.);
    assert!(slide.progress(middle, normal) < 1.);

    cx.update(crate::accessibility::policy::tests::reduce);
    let reduced = slide_duration(SLIDE);
    assert_eq!(slide.at(middle, reduced), 10.);
    assert_eq!(slide.progress(middle, reduced), 1.);
    let reversed = slide.toward(-10., middle, reduced);
    assert_eq!(reversed.at(middle, reduced), -10.);
    assert_eq!(reversed.progress(middle, reduced), 1.);
}
