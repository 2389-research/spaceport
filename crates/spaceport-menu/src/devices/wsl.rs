//! The Add WSL form: this machine's distributions, one of which the form adds
//! once Herdr answers inside it, and the question that forgets a saved one.
//! Nothing here installs Herdr, edits a distribution, or boots one the user
//! did not pick.

use crate::chrome::Text;
use crate::devices::{Field, Submit, button, field, form, form_body, form_footer};
use gpui::prelude::*;
use gpui::*;
use spaceport_settings::{Theme, corners};
use spaceport_widgets::toggles::radio;

/// One distribution the form lists.
pub struct Distro {
    /// Its own name, which is data.
    pub name: String,
    /// Whether it is already a device, so it cannot be added again.
    pub added: bool,
    /// Whether it is the one the form would add.
    pub chosen: bool,
}

impl Distro {
    /// Whether the form can still add it.
    fn addable(&self) -> bool {
        !self.added
    }
}

/// What the form says in place of a list of distributions.
pub enum List {
    /// A dimmed line where the list would be: still looking, or none
    /// installed.
    Note(String),
    /// The list failed, in the warning colour.
    Unavailable(String),
    /// Each distribution, with whether it is already a device.
    Distros(Vec<Distro>),
}

/// The form, as the app prepared it.
pub struct Form {
    /// The sentence above the list.
    pub intro: String,
    /// What the list says.
    pub list: List,
    /// The word a distribution that is already a device shows.
    pub added_label: String,
    /// The session the distribution's Herdr runs in.
    pub session: Field,
    /// Why the last add failed, in the dialog's words.
    pub error: Option<String>,
    /// What the check under way is doing, if one is.
    pub status: Option<String>,
    /// The footer's button.
    pub submit: Submit,
    /// The colour a failure is written in, which the app takes from the
    /// theme's contrast rules.
    pub danger: Rgba,
}

/// What pressing something on the form asks the window to do.
pub enum Action {
    /// Choose the distribution this row names.
    Pick(String),
    /// Add the chosen distribution.
    Submit,
}

/// The distribution the arrow keys step to from `chosen`: only those that can
/// still be added, as a press on a row allows, wrapping at either end.
pub fn stepped<'a>(distros: &'a [Distro], chosen: Option<&str>, forward: bool) -> Option<&'a str> {
    let addable: Vec<&Distro> = distros.iter().filter(|distro| distro.addable()).collect();
    if addable.is_empty() {
        return None;
    }
    let at = chosen.and_then(|chosen| addable.iter().position(|distro| distro.name == chosen));
    let next = match (at, forward) {
        (None, false) => addable.len() - 1,
        (None, true) => 0,
        (Some(at), false) => (at + addable.len() - 1) % addable.len(),
        (Some(at), true) => (at + 1) % addable.len(),
    };
    addable.get(next).map(|distro| distro.name.as_str())
}

/// The form under the dialog's frame. `interact` takes each painted row and
/// the footer's button back and wires what pressing them does.
pub fn body(
    text: &Text,
    theme: &Theme,
    values: Form,
    mut interact: impl FnMut(Action, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let Form {
        intro,
        list,
        added_label,
        session,
        error,
        status,
        submit,
        danger,
    } = values;
    let mut distros = div()
        .debug_selector(|| "wsl-setup-distros".into())
        .flex()
        .flex_col()
        .gap(px(2.));
    match list {
        List::Note(note) => distros = distros.child(div().text_color(rgb(theme.muted)).child(note)),
        List::Unavailable(reason) => {
            distros = distros.child(div().text_color(danger).child(reason))
        }
        List::Distros(rows) => {
            for (index, row) in rows.iter().enumerate() {
                let entry = distro(theme, text, row, &added_label, index);
                // A distribution already saved is a row to read, not to press.
                distros = distros.child(match row.added {
                    true => entry,
                    false => interact(Action::Pick(row.name.clone()), entry),
                });
            }
        }
    }
    let mut body = form_body("wsl-setup-body")
        .child(
            div()
                .flex_none()
                .text_color(rgb(theme.subtext()))
                .child(intro),
        )
        .child(distros)
        .child(field(session));
    if let Some(error) = error {
        body = body.child(
            div()
                .debug_selector(|| "wsl-setup-error".into())
                .flex_none()
                .text_color(danger)
                .child(error),
        );
    }
    if let Some(status) = status {
        body = body.child(
            div()
                .debug_selector(|| "wsl-setup-status".into())
                .flex_none()
                .text_color(rgb(theme.muted))
                .child(status),
        );
    }
    let footer = form_footer(theme, None).child(interact(Action::Submit, button(theme, &submit)));
    form(body, footer)
}

/// One distribution's row: the mark saying whether the form would add it, its
/// name, and the word saying it is already a device.
fn distro(
    theme: &Theme,
    text: &Text,
    row: &Distro,
    added_label: &str,
    index: usize,
) -> Stateful<Div> {
    div()
        .id(("wsl-distro", index))
        .debug_selector(move || format!("wsl-distro-{index}"))
        .px(px(8.))
        .py(px(6.))
        .rounded(px(corners::CONTROL))
        .flex()
        .items_center()
        .gap(px(8.))
        .when(row.chosen, |row| row.bg(rgb(theme.active)))
        .when(row.added, |row| row.text_color(rgb(theme.muted)))
        .when(!row.added, |row| {
            row.cursor_pointer()
                .hover(|hovered| hovered.bg(rgb(theme.active)))
        })
        .child(radio(theme, text.size + 2., row.chosen))
        .child(div().flex_1().truncate().child(row.name.clone()))
        .when(row.added, |row| row.child(added_label.to_owned()))
}

/// The question that forgets a saved distribution, as the app prepared it.
pub struct Question {
    /// What the question asks.
    pub title: String,
    /// What it explains, worded with the distribution's name.
    pub note: String,
    /// The button that leaves it saved.
    pub cancel: String,
    /// The button that forgets it.
    pub remove: String,
    /// The colour the removal is written in.
    pub danger: Rgba,
}

/// What pressing either of the question's buttons asks the window to do.
pub enum Removal {
    /// Leave the distribution saved.
    Cancel,
    /// Forget it.
    Confirm,
}

/// The removal question. `interact` takes each painted button back and wires
/// what pressing it does.
pub fn removal(
    theme: &Theme,
    question: Question,
    mut interact: impl FnMut(Removal, Stateful<Div>) -> Stateful<Div>,
) -> Div {
    let Question {
        title,
        note,
        cancel,
        remove,
        danger,
    } = question;
    div()
        .debug_selector(|| "remove-wsl".into())
        .flex()
        .flex_col()
        .p(px(8.))
        .gap(px(12.))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
        .child(div().text_color(rgb(theme.subtext())).child(note))
        .child(
            div()
                .flex()
                .justify_end()
                .gap(px(8.))
                .child(interact(
                    Removal::Cancel,
                    div()
                        .id("remove-wsl-cancel")
                        .p(px(6.))
                        .cursor_pointer()
                        .child(cancel),
                ))
                .child(interact(
                    Removal::Confirm,
                    div()
                        .id("remove-wsl-submit")
                        .debug_selector(|| "remove-wsl-submit".into())
                        .p(px(6.))
                        .rounded(px(corners::CONTROL))
                        .bg(rgb(theme.active))
                        .text_color(danger)
                        .cursor_pointer()
                        .child(remove),
                )),
        )
}
