//! Chrome that mirrors in a right-to-left language (docs/i18n.md, Phase 4).
//! GPUI lays everything out left to right, so the chrome names its sides
//! logically instead: the start is the left in English and the right in
//! Hebrew, and the end the other side. [`Logical`] gives every styled element
//! start and end padding, margins, insets, borders, corners, rows, and text
//! alignment; the functions here mirror a popover's anchor, a slide's
//! offset, and an icon that points somewhere.
//!
//! The rules themselves are `spaceport-sidebar`'s `direction`, keyed there on a
//! `bool` because a row's painter cannot read a locale: every method here reads
//! [`is_rtl`] and delegates, so the app and a row cannot mirror differently.
//!
//! The direction is the shown locale's, published with [`set`] by the app's
//! i18n as it switches the catalog, so a locale switch mirrors whatever is
//! built after it. It is the process's, as the catalog is, because an element is
//! built where there is no app context; each test keeps its own while
//! `test-support` is on, as the app's catalog does in its tests, so one test's
//! pseudo-locale cannot mirror another's layout. The terminal grid and its
//! content never mirror: panes, splits, and tab strips are the daemon's
//! geometry, and a terminal draws its cells left to right whatever the interface
//! reads.

use gpui::{AbsoluteLength, Anchor, DefiniteLength, Length, Styled, TextAlign};
pub use spaceport_i18n::Direction;
use spaceport_sidebar::direction as mirror;

// `spaceport_sidebar::direction` is named as a module rather than imported as a
// trait: the two `Logical` traits would otherwise both answer a bare
// `element.ps(..)` call in this crate.

/// Where the direction is kept: one flag for the process, as the app's catalog
/// is, or one per thread while `test-support` is on.
mod store {
    #[cfg(not(any(test, feature = "test-support")))]
    static RTL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    #[cfg(not(any(test, feature = "test-support")))]
    pub(super) fn set(right_to_left: bool) {
        RTL.store(right_to_left, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(not(any(test, feature = "test-support")))]
    pub(super) fn get() -> bool {
        RTL.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[cfg(any(test, feature = "test-support"))]
    thread_local! {
        static RTL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(super) fn set(right_to_left: bool) {
        RTL.set(right_to_left);
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(super) fn get() -> bool {
        RTL.get()
    }
}

/// Publishes the shown locale's direction: the app's i18n calls it as it
/// switches the catalog, so layout built after the switch mirrors.
pub fn set(direction: Direction) {
    store::set(direction == Direction::Rtl);
}

/// The direction the chrome lays out in now.
pub fn current() -> Direction {
    match store::get() {
        true => Direction::Rtl,
        false => Direction::Ltr,
    }
}

/// Whether the chrome lays out right to left now.
pub fn is_rtl() -> bool {
    current() == Direction::Rtl
}

/// Start and end sides for any styled element. Each picks the physical side
/// when it is called, so call them while building an element, not ahead.
pub trait Logical: Styled + Sized {
    /// Children from the start to the end: a row, reversed right to left.
    fn row(self) -> Self {
        mirror::Logical::row(self, is_rtl())
    }

    /// Padding on the side a line of text starts.
    fn ps(self, length: impl Clone + Into<DefiniteLength>) -> Self {
        mirror::Logical::ps(self, is_rtl(), length)
    }

    /// Padding on the side a line of text ends.
    fn pe(self, length: impl Clone + Into<DefiniteLength>) -> Self {
        mirror::Logical::pe(self, is_rtl(), length)
    }

    fn ms(self, length: impl Clone + Into<Length>) -> Self {
        mirror::Logical::ms(self, is_rtl(), length)
    }

    fn me(self, length: impl Clone + Into<Length>) -> Self {
        mirror::Logical::me(self, is_rtl(), length)
    }

    /// An absolutely placed element's distance from its container's start.
    fn inset_s(self, length: impl Clone + Into<Length>) -> Self {
        mirror::Logical::inset_s(self, is_rtl(), length)
    }

    /// An absolutely placed element's distance from its container's end.
    fn inset_e(self, length: impl Clone + Into<Length>) -> Self {
        mirror::Logical::inset_e(self, is_rtl(), length)
    }

    fn border_s(self, width: impl Clone + Into<AbsoluteLength>) -> Self {
        mirror::Logical::border_s(self, is_rtl(), width)
    }

    fn border_e(self, width: impl Clone + Into<AbsoluteLength>) -> Self {
        mirror::Logical::border_e(self, is_rtl(), width)
    }

    /// Text set against the side it starts from.
    fn text_start(self) -> Self {
        self.text_align(text_start())
    }

    /// Text set against the side it ends on, as numbers in a column are.
    fn text_end(self) -> Self {
        mirror::Logical::text_end(self, is_rtl())
    }
}

impl<E: Styled> Logical for E {}

/// The alignment of a line set from its start, for text painted by hand.
pub fn text_start() -> TextAlign {
    spaceport_sidebar::direction::text_start(is_rtl())
}

/// `anchor` as written for a left-to-right layout, mirrored when the chrome
/// runs right to left: a menu that hangs from its button's left corner
/// hangs from its right one.
pub fn anchor(anchor: Anchor) -> Anchor {
    if !is_rtl() {
        return anchor;
    }
    match anchor {
        Anchor::TopLeft => Anchor::TopRight,
        Anchor::TopRight => Anchor::TopLeft,
        Anchor::BottomLeft => Anchor::BottomRight,
        Anchor::BottomRight => Anchor::BottomLeft,
        Anchor::LeftCenter => Anchor::RightCenter,
        Anchor::RightCenter => Anchor::LeftCenter,
        centered @ (Anchor::TopCenter | Anchor::BottomCenter) => centered,
    }
}

/// A horizontal distance toward the end: positive to the right in a
/// left-to-right layout, to the left in a right-to-left one. A slide that
/// leaves toward the start moves by `toward_end(-width)`.
pub fn toward_end(x: f32) -> f32 {
    match current() {
        Direction::Ltr => x,
        Direction::Rtl => -x,
    }
}

/// `path`'s mirror image when the chrome runs right to left and the icon
/// points along the line: a disclosure chevron, Back and Forward, a panel on
/// one side. Every other icon is drawn as it is. The tables live in
/// `spaceport-sidebar` with the rules that read them, so a row mirrors an arrow
/// exactly as the chrome does.
pub fn icon(shown: &'static str) -> &'static str {
    spaceport_sidebar::direction::icon(shown, is_rtl())
}

/// A text arrow that points along the line, such as a folded row's triangle or
/// a breadcrumb's separator, mirrored right to left.
pub fn glyph(shown: &'static str) -> &'static str {
    spaceport_sidebar::direction::glyph(shown, is_rtl())
}

#[cfg(test)]
mod tests;
