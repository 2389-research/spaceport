//! The body a menu's form page draws: the dialog a workspace action asks for,
//! and the rename form its tab and pane menus ask for.
//!
//! A form is a heading, the line under it that says what it asks, the draft the
//! app built, and two buttons. The app words every line, owns the draft field
//! and says what pressing a button does; what is here is the layout, the paint
//! and the element ids its tests and native checks find the buttons by, taken
//! back through `interact` the way [`crate::rows`] gives a row back.

use crate::chrome::Text;
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};
use spaceport_widgets::direction::Logical;

/// Which of a form's two buttons `interact` is handed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// The one that closes the form without sending it.
    Cancel,
    /// The one that sends it.
    Submit,
}

/// What a form's two buttons say, as the app worded them. A submit says what
/// it does now, so a form whose answer is on its way reads "Renaming".
#[derive(Clone, Debug, PartialEq)]
pub struct Buttons {
    pub cancel: String,
    pub submit: String,
}

/// The heading over a rename form, the hint under it, and the refusal the last
/// submit earned.
#[derive(Clone, Debug, PartialEq)]
pub struct Rename {
    /// What the two buttons say.
    pub buttons: Buttons,
    /// The reason the last submit was refused, drawn under the form.
    pub error: Option<String>,
    /// The line under the heading that says what the form asks for; a page
    /// without one passes none.
    pub hint: Option<String>,
    /// The tail of the form's element ids: `tab` names `tab-rename-cancel` and
    /// `tab-rename-submit`.
    pub selector: &'static str,
    /// The heading.
    pub title: String,
}

/// The dialog a workspace action asks for: what it is for, the workspace it
/// names under its heading, and the refusal the last submit earned.
#[derive(Clone, Debug, PartialEq)]
pub struct Dialog {
    /// What the two buttons say.
    pub buttons: Buttons,
    /// The reason the last submit was refused, drawn in the danger colour.
    pub error: Option<String>,
    /// The artwork beside the heading, as an asset path.
    pub icon: Option<&'static str>,
    /// The line under the heading that explains what the dialog asks for.
    pub note: String,
    /// The workspace the dialog names under its heading.
    pub target: String,
    /// The heading.
    pub title: String,
}

impl Rename {
    /// The element id and debug selector of one of the form's two buttons.
    fn button(&self, button: Button) -> String {
        let tail = match button {
            Button::Cancel => "cancel",
            Button::Submit => "submit",
        };
        format!("{}-rename-{tail}", self.selector)
    }
}

/// `body` with one rename form in it: its heading, the hint under it, the draft
/// `field` the app built, and its two buttons, which `interact` takes back
/// wired with what pressing them does.
pub fn rename(
    body: Div,
    hosts: &Rename,
    theme: &Theme,
    field: Option<AnyElement>,
    mut interact: impl FnMut(Button, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let form = body
        .p(px(8.))
        .gap(px(12.))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(hosts.title.clone()),
        )
        .children(
            hosts
                .hint
                .as_ref()
                .map(|hint| div().text_color(rgb(theme.subtext())).child(hint.clone())),
        )
        .children(field);
    let mut press = |button: Button, words: &str| {
        let id = hosts.button(button);
        interact(
            button,
            div()
                .id(id.clone())
                .debug_selector(move || id)
                .p(px(6.))
                .cursor_pointer()
                .when(button == Button::Submit, |button| {
                    button.rounded(px(corners::CONTROL)).bg(rgb(theme.active))
                })
                .child(words.to_owned()),
        )
    };
    let mut form = form.child(
        div()
            .row()
            .justify_end()
            .gap(px(8.))
            .child(press(Button::Cancel, &hosts.buttons.cancel))
            .child(press(Button::Submit, &hosts.buttons.submit)),
    );
    // No width of its own, so the labels set the panel's width and the message
    // wraps inside it.
    form = form.children(
        hosts
            .error
            .as_ref()
            .map(|error| div().w_0().min_w_full().p(px(8.)).child(error.clone())),
    );
    form
}

/// A workspace action's dialog: its header, the body each action asks for, and
/// the footer whose submit button sends it. `field` is the draft the action
/// asked for, `danger` the refusal colour, and `interact` takes each footer
/// button back wired with what pressing it does.
pub fn dialog(
    hosts: &Dialog,
    text: &Text,
    theme: &Theme,
    danger: Rgba,
    field: Option<AnyElement>,
    mut interact: impl FnMut(Button, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let mut body = div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .child(
            div()
                .text_color(rgb(theme.subtext()))
                .child(hosts.note.clone()),
        )
        .children(field);
    if let Some(error) = &hosts.error {
        body = body.child(
            div()
                .debug_selector(|| "dialog-error".into())
                .rounded(px(corners::CONTROL))
                .bg(rgb(theme.active))
                .px(px(10.))
                .py(px(6.))
                .text_color(danger)
                .child(error.clone()),
        );
    }
    let button = |id: &'static str| {
        div()
            .id(id)
            .debug_selector(move || id.into())
            .px(px(12.))
            .py(px(6.))
            .rounded(px(corners::CONTROL))
            .border_1()
            .cursor_pointer()
    };
    let cancel = interact(
        Button::Cancel,
        button("dialog-cancel")
            .role(Role::Button)
            .aria_label(hosts.buttons.cancel.clone())
            .border_color(rgb(theme.active))
            .hover(|button| button.bg(rgb(theme.active)))
            .child(hosts.buttons.cancel.clone()),
    );
    let submit = interact(
        Button::Submit,
        button("dialog-submit")
            .role(Role::DefaultButton)
            .aria_label(hosts.buttons.submit.clone())
            .border_color(rgb(theme.foreground))
            .bg(rgb(theme.active))
            .text_color(rgb(theme.foreground))
            .child(hosts.buttons.submit.clone())
            .hover(|button| button.bg(rgb(theme.active))),
    );
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .debug_selector(|| "dialog-header".into())
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(16.))
                .py(px(12.))
                .border_b_1()
                .border_color(rgb(theme.active))
                .children(hosts.icon.map(|icon| {
                    svg()
                        .path(icon)
                        .size(px(16.))
                        .flex_none()
                        .text_color(rgb(theme.muted))
                }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .debug_selector(|| "dialog-title".into())
                                .text_size(px(text.size * 1.35))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(hosts.title.clone()),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_color(rgb(theme.muted))
                                .child(hosts.target.clone()),
                        ),
                ),
        )
        .child(body)
        .child(
            div()
                .debug_selector(|| "dialog-footer".into())
                .flex()
                .justify_end()
                .gap(px(8.))
                .px(px(16.))
                .py(px(12.))
                .border_t_1()
                .border_color(rgb(theme.active))
                .child(cancel)
                .child(submit),
        )
}

#[cfg(test)]
mod tests;
