//! The chrome's brief motion for what comes and goes in a window: an element
//! appearing, timed by `spaceport-motion` so it keeps the app's clock and the
//! QA > Slow Animations switch.

use gpui::{AnimationElement, AnimationExt, ElementId, IntoElement, Styled, px};
use spaceport_motion::{ENTER, duration};

/// How far a dropdown drops into place as it appears.
const DROP: f32 = 4.;

/// A dropdown or small popover appearing: it fades in while dropping a few
/// pixels into place, on the same curve as `spaceport_motion::progress`. `id`
/// names it; GPUI plays the motion again each time an element of that id
/// appears anew. A test build (`test-support`) sees it in place, so geometry
/// checks need not wait it out.
pub fn pop_in<E: IntoElement + Styled + 'static>(
    element: E,
    id: impl Into<ElementId>,
) -> AnimationElement<E> {
    element.with_animation(
        id,
        gpui::Animation::new(duration(ENTER)).with_easing(|k| 1. - (1. - k).powi(3)),
        |element, k| {
            let k = if cfg!(feature = "test-support") {
                1.
            } else {
                k
            };
            element.opacity(k).mt(px(-(1. - k) * DROP))
        },
    )
}
