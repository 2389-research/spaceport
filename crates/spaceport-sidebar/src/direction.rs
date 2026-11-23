//! The mirroring rules a row follows in a right-to-left language, keyed on a
//! `bool` rather than on the process's locale: a crate cannot read the shown
//! language, so the app passes `right_to_left` in as an input like the rest.
//! GPUI lays everything out left to right, so sides are named logically — the
//! start is the left in English and the right in Hebrew, and the end the other
//! side. [`Logical`] gives every styled element start and end padding, margins,
//! insets, borders, corners, rows, and text alignment; [`glyph`] and [`icon`]
//! mirror an arrow that points along the line.
//!
//! The app's `direction` module is the same policy with the locale read for it,
//! so the two clients of a row cannot drift.

use gpui::{AbsoluteLength, DefiniteLength, Length, Styled, TextAlign};

/// Start and end sides for any styled element. Each picks the physical side
/// when it is called, so call them while building an element, not ahead.
/// `right_to_left` is the shown locale's direction, passed in by the app.
///
/// The app's `crate::direction::Logical` has the same methods without the
/// flag; this one is `spaceport-sidebar`'s, so a row painter names the side
/// without reading any process-wide state.
pub trait Logical: Styled + Sized {
    /// Children from the start to the end: a row, reversed right to left.
    fn row(self, right_to_left: bool) -> Self {
        if right_to_left {
            self.flex().flex_row_reverse()
        } else {
            self.flex().flex_row()
        }
    }

    /// Padding on the side a line of text starts.
    fn ps(self, right_to_left: bool, length: impl Clone + Into<DefiniteLength>) -> Self {
        if right_to_left {
            self.pr(length)
        } else {
            self.pl(length)
        }
    }

    /// Padding on the side a line of text ends.
    fn pe(self, right_to_left: bool, length: impl Clone + Into<DefiniteLength>) -> Self {
        if right_to_left {
            self.pl(length)
        } else {
            self.pr(length)
        }
    }

    fn ms(self, right_to_left: bool, length: impl Clone + Into<Length>) -> Self {
        if right_to_left {
            self.mr(length)
        } else {
            self.ml(length)
        }
    }

    fn me(self, right_to_left: bool, length: impl Clone + Into<Length>) -> Self {
        if right_to_left {
            self.ml(length)
        } else {
            self.mr(length)
        }
    }

    /// An absolutely placed element's distance from its container's start.
    fn inset_s(self, right_to_left: bool, length: impl Clone + Into<Length>) -> Self {
        if right_to_left {
            self.right(length)
        } else {
            self.left(length)
        }
    }

    /// An absolutely placed element's distance from its container's end.
    fn inset_e(self, right_to_left: bool, length: impl Clone + Into<Length>) -> Self {
        if right_to_left {
            self.left(length)
        } else {
            self.right(length)
        }
    }

    fn border_s(self, right_to_left: bool, width: impl Clone + Into<AbsoluteLength>) -> Self {
        if right_to_left {
            self.border_r(width)
        } else {
            self.border_l(width)
        }
    }

    fn border_e(self, right_to_left: bool, width: impl Clone + Into<AbsoluteLength>) -> Self {
        if right_to_left {
            self.border_l(width)
        } else {
            self.border_r(width)
        }
    }

    /// Rounds the two corners on the end side.
    fn rounded_e(self, right_to_left: bool, radius: impl Clone + Into<AbsoluteLength>) -> Self {
        if right_to_left {
            self.rounded_l(radius)
        } else {
            self.rounded_r(radius)
        }
    }

    /// Text set against the side it starts from.
    fn text_start(self, right_to_left: bool) -> Self {
        self.text_align(text_start(right_to_left))
    }

    /// Text set against the side it ends on, as numbers in a column are.
    fn text_end(self, right_to_left: bool) -> Self {
        self.text_align(match right_to_left {
            false => TextAlign::Right,
            true => TextAlign::Left,
        })
    }
}

impl<E: Styled> Logical for E {}

/// The alignment of a line set from its start, for text painted by hand.
pub fn text_start(right_to_left: bool) -> TextAlign {
    match right_to_left {
        false => TextAlign::Left,
        true => TextAlign::Right,
    }
}

/// Declares each icon or text arrow that points along the line once, with its
/// mirror, as `$function` and the `$all` table the tests read. The tables are
/// built in every profile: the app's own tests read them, and they are the
/// same pairs the match below compiles to.
macro_rules! mirrored {
    ($(#[$doc:meta])* $function:ident, $all:ident { $($one:literal <=> $other:literal,)+ }) => {
        $(#[$doc])*
        pub fn $function(shown: &'static str, right_to_left: bool) -> &'static str {
            if !right_to_left {
                return shown;
            }
            match shown {
                $(
                    $one => $other,
                    $other => $one,
                )+
                _ => shown,
            }
        }

        /// Each pair `$function` swaps, as `(shown, mirrored)`.
        pub const $all: &[(&str, &str)] = &[$(($one, $other),)+];
    };
}

mirrored! {
    /// `path`'s mirror image when the chrome runs right to left and the icon
    /// points along the line: a disclosure chevron, Back and Forward, a
    /// panel on one side. Every other icon is drawn as it is.
    icon, DIRECTIONAL_ICONS {
        // keep-sorted start
        "icons/arrow-left.svg" <=> "icons/arrow-right.svg",
        "icons/chevron-left.svg" <=> "icons/chevron-right.svg",
        "icons/panel-left.svg" <=> "icons/panel-right.svg",
        // keep-sorted end
    }
}

mirrored! {
    /// A text arrow that points along the line, such as a folded row's
    /// triangle or a breadcrumb's separator, mirrored right to left. Text
    /// the shaper lays out in a right-to-left run mirrors `›` itself; these
    /// are drawn alone, in a left-to-right paragraph.
    glyph, DIRECTIONAL_GLYPHS {
        // keep-sorted start
        "\u{203a}" <=> "\u{2039}",
        "\u{2192}" <=> "\u{2190}",
        "\u{25b8}" <=> "\u{25c2}",
        // keep-sorted end
    }
}

#[cfg(test)]
mod tests;
