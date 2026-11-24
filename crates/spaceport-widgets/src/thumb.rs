//! Where a thumb sits over the content it scrolls, and how far dragging it
//! scrolls: the same math along either axis, shared by a group's tab strip and
//! the Settings panes' scrollbars.

/// The narrowest the thumb draws, so a strip of many tabs keeps one to grab.
const MIN_THUMB: f32 = 24.;

/// Where a strip's thumb sits, from the strip's left edge. The math is the
/// same along either axis, so the review's vertical scrollbar uses it too.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thumb {
    pub left: f32,
    pub width: f32,
}

impl Thumb {
    /// The thumb for a strip `viewport` wide that scrolls `max` further,
    /// scrolled `scrolled` of the way; none when its tabs fit.
    pub fn of(viewport: f32, max: f32, scrolled: f32) -> Option<Self> {
        if viewport <= 0. || max < 0.5 {
            return None;
        }
        let width =
            (viewport * viewport / (viewport + max)).clamp(MIN_THUMB.min(viewport), viewport);
        let left = (scrolled / max).clamp(0., 1.) * (viewport - width);
        Some(Self { left, width })
    }

    /// How far a strip scrolls with this thumb moved to `left`.
    pub fn scrolled_at(self, viewport: f32, max: f32, left: f32) -> f32 {
        let travel = viewport - self.width;
        if travel <= 0. {
            return 0.;
        }
        (left / travel).clamp(0., 1.) * max
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn a_strip_whose_tabs_fit_draws_no_thumb() {
        assert_eq!(Thumb::of(400., 0., 0.), None);
        assert_eq!(Thumb::of(0., 100., 0.), None);
    }

    #[test]
    fn the_thumb_spans_the_share_shown_and_travels_with_the_scroll() {
        // Half the tabs show, so the thumb is half the strip.
        let start = Thumb::of(400., 400., 0.).unwrap();
        assert_eq!(
            start,
            Thumb {
                left: 0.,
                width: 200.
            }
        );
        let end = Thumb::of(400., 400., 400.).unwrap();
        assert_eq!(
            end,
            Thumb {
                left: 200.,
                width: 200.
            }
        );
        // Scrolled past either end, as a bounce can, it stays in its track.
        let past = Thumb::of(400., 400., 900.).unwrap();
        assert_eq!(past.left, 200.);
    }

    #[test]
    fn many_tabs_keep_a_thumb_wide_enough_to_grab() {
        let thumb = Thumb::of(200., 100_000., 0.).unwrap();
        assert_eq!(thumb.width, MIN_THUMB);
    }

    #[test]
    fn dragging_the_thumb_scrolls_in_proportion_and_stops_at_the_ends() {
        let thumb = Thumb {
            left: 0.,
            width: 200.,
        };
        assert_eq!(thumb.scrolled_at(400., 400., 100.), 200.);
        assert_eq!(thumb.scrolled_at(400., 400., -50.), 0.);
        assert_eq!(thumb.scrolled_at(400., 400., 900.), 400.);
    }
}
