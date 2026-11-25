//! How the crate reaches what only the app can do: paints the text a row's
//! label is (which the app's headless and native probes record) and draws a
//! moving mark in the window's own layer. A row painter needs both and can own
//! neither. The words the crate shows are its own, read from `spaceport-i18n`.
//!
//! The seams are function pointers, not closures: neither has state, so the app
//! hands over two `fn` items and [`Painters`] stays `Copy`, which is what keeps
//! [`RowContext`](crate::row::RowContext) free of another lifetime parameter.
//!
//! **A generic method can never be the seam.** `Cell::row` reaches its layout
//! through `&'static dyn RowLayout`, whose methods take `&RowContext<'_>` and
//! return concrete elements; a `RowLayout<L: Label>` design would force an
//! erased element at the trait boundary anyway or monomorphize every layout
//! type. Putting the seams in the struct is what keeps the trait object-safe,
//! so a later stage must not move them out to a generic parameter. If one ever
//! needs to capture state, its field becomes a reference to a closure and
//! `RowContext` gains a borrow it already has; the call sites do not change.

use gpui::{AnyElement, IntoElement, SharedString};
use std::rc::Rc;

/// What only the app can paint, by value so a row's context stays `Copy`-shaped
/// and every layout can carry it.
#[derive(Clone, Copy)]
pub struct Painters {
    /// The element a label is painted as.
    pub text: fn(SharedString) -> AnyElement,
    /// A moving mark, `size` across at the opacity `fade` of its row, drawn
    /// where the window can compose it.
    pub mark_layer: fn(f32, f32, Rc<dyn Fn() -> AnyElement>) -> AnyElement,
}

impl Painters {
    /// The element for one line of a row's text.
    pub fn label(&self, text: impl Into<SharedString>) -> AnyElement {
        (self.text)(text.into())
    }

    /// `build`'s mark, drawn where the window can compose it moving and faded
    /// to its row's own opacity, which a layered mark cannot inherit.
    pub fn layered(
        &self,
        size: f32,
        fade: f32,
        build: impl Fn() -> AnyElement + 'static,
    ) -> AnyElement {
        (self.mark_layer)(size, fade, Rc::new(build))
    }
}

/// The seams with nothing behind them: text painted as itself, and a mark drawn
/// where it sits. For the crate's own tests and any consumer with no window.
pub const PLAIN: Painters = Painters {
    text: plain_text,
    mark_layer: in_place,
};

fn plain_text(text: SharedString) -> AnyElement {
    text.into_any_element()
}

/// A mark with no layer to draw in: built where it sits, so its row's own
/// opacity fades it and `size` and `fade` have nothing left to do.
fn in_place(_size: f32, _fade: f32, build: Rc<dyn Fn() -> AnyElement>) -> AnyElement {
    build()
}

#[cfg(test)]
mod tests;
