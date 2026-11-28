//! The Add Device form: the machine to reach, the name it takes in the
//! catalog, and the Herdr session it opens, with what the request that was
//! submitted is doing.

use crate::devices::{Field, Submit, button, field, form, form_body, form_footer};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::Theme;

/// The form, as the app prepared it.
pub struct Form {
    /// The sentence above the fields.
    pub intro: String,
    /// The values the form asks for, top to bottom.
    pub fields: Vec<Field>,
    /// Why the last add failed, in the dialog's words.
    pub error: Option<String>,
    /// What the submitted request is doing, if one was sent.
    pub status: Option<String>,
    /// The footer's button.
    pub submit: Submit,
    /// The colour a failure is written in, which the app takes from the
    /// theme's contrast rules.
    pub danger: Rgba,
}

/// What pressing the form's button asks the window to do.
pub enum Action {
    /// Add the machine the fields name.
    Submit,
}

/// The form under the dialog's frame. `interact` takes the painted button back
/// and wires what pressing it does.
pub fn body(
    theme: &Theme,
    values: Form,
    mut interact: impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let Form {
        intro,
        fields,
        error,
        status,
        submit,
        danger,
    } = values;
    let mut body = form_body("device-setup-body").child(
        div()
            .flex_none()
            .text_color(rgb(theme.subtext()))
            .child(intro),
    );
    for value in fields {
        body = body.child(field(value));
    }
    if let Some(error) = error {
        body = body.child(div().flex_none().text_color(danger).child(error));
    }
    if let Some(status) = status {
        body = body.child(
            div()
                .debug_selector(|| "device-setup-status".into())
                .flex_none()
                .text_color(rgb(theme.muted))
                .child(status),
        );
    }
    let footer = form_footer(theme, Some("device-setup-footer"))
        .child(interact(Action::Submit, button(theme, &submit)));
    form(body, footer)
}
