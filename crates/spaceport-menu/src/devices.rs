//! The Add Device dialog, below the window: the frame its sources share, the
//! form that adds a machine by hand, and the form that adds a WSL distribution
//! or forgets one.
//!
//! A page here paints only what the app prepared: every word it says, the
//! debug names its tests read, the colour a failure is written in, and each
//! value it asks for as either the window's own field or the text that field
//! held when its request was sent. What pressing something does stays the
//! window's: every control a page paints comes back through one `interact`
//! closure over that part's closed set of actions, which the app wires, as
//! [`crate::rows::list`] does.

pub mod add_device;
pub mod dialog;
pub mod wsl;

use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};

/// One value a form asks for: the word it asks in, and what draws under it.
pub struct Field {
    /// What the form calls the value.
    pub label: String,
    /// What draws under the label.
    pub value: Value,
}

/// What draws under a [`Field`]'s label.
pub enum Value {
    /// The window's own field, which only the app builds and focuses.
    Input(AnyElement),
    /// The text that field held when its request was sent, in its place: the
    /// request carries what was typed, not what the fields hold now.
    Typed(String),
}

/// A form's footer button, as the app prepared it.
pub struct Submit {
    /// The button's debug selector, which also names its element.
    pub selector: String,
    /// What it says.
    pub label: String,
    /// Whether pressing it does anything yet.
    pub enabled: bool,
}

/// `field`'s row: its label over what draws under it.
pub(crate) fn field(field: Field) -> Div {
    let row = div()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(field.label);
    match field.value {
        Value::Input(input) => row.child(input),
        Value::Typed(text) => row.child(text),
    }
}

/// The footer's button, dimmed and inert until `enabled`, before the window
/// wires what pressing it does.
pub(crate) fn button(theme: &Theme, submit: &Submit) -> Stateful<Div> {
    let selector = submit.selector.clone();
    div()
        .id(selector.clone())
        .debug_selector(move || selector)
        .p(px(8.))
        .rounded(px(corners::CONTROL))
        .bg(rgb(theme.active))
        .when(submit.enabled, |button| {
            button.cursor_pointer().hover(|hovered| {
                hovered.bg(rgb(theme.active).blend(rgba((theme.foreground << 8) | 0x20)))
            })
        })
        .when(!submit.enabled, |button| {
            button.text_color(rgb(theme.muted))
        })
        .child(submit.label.clone())
}

/// The room a dialog's body and footer share, under the frame.
pub(crate) fn form(body: impl IntoElement, footer: impl IntoElement) -> Div {
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(body)
        .child(footer)
}

/// A form's body: the scrolling room its values are laid out in.
pub(crate) fn form_body(selector: &'static str) -> Stateful<Div> {
    div()
        .id(selector)
        .debug_selector(move || selector.into())
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(16.))
        .flex()
        .flex_col()
        .gap(px(12.))
}

/// A form's footer: the rule above the button that submits it.
pub(crate) fn form_footer(theme: &Theme, selector: Option<&'static str>) -> Div {
    div()
        .flex_none()
        .p(px(16.))
        .border_t_1()
        .border_color(rgb(theme.active))
        .flex()
        .justify_end()
        .gap(px(8.))
        .when_some(selector, |footer, selector| {
            footer.debug_selector(move || selector.into())
        })
}

#[cfg(test)]
mod tests;
