#![allow(clippy::unwrap_used)]
//! What a form draws, and what its two buttons report: the heading and the hint
//! over the draft, the draft the app built, the refusal under it, and the
//! footer `interact` takes back.

use super::*;
use core::prelude::v1::test;
use std::{cell::Cell, rc::Rc};

fn text() -> Text {
    Text {
        font: Font::default(),
        line_height: 20.,
        size: 13.,
    }
}

fn words() -> Buttons {
    Buttons {
        cancel: "Cancel".into(),
        submit: "Rename".into(),
    }
}

/// What the theme's danger colour is, as a form is handed it prepared.
fn danger() -> Rgba {
    rgb(0xdd3322)
}

/// The draft the app builds: a field of its own, which a test finds by the
/// selector it gives it.
fn draft() -> AnyElement {
    div()
        .id("draft")
        .debug_selector(|| "draft".into())
        .child("main")
        .into_any_element()
}

/// A rename form as the tab and pane menus draw it, with a draft standing in
/// for the field the app builds.
struct RenameHarness {
    pressed: Rc<Cell<Option<Button>>>,
    /// Whether the form says what it asks and why it was refused.
    refused: bool,
}

impl Render for RenameHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let pressed = self.pressed.clone();
        let hosts = Rename {
            buttons: words(),
            error: self.refused.then(|| "that name is taken".to_owned()),
            hint: self.refused.then(|| "Type a new name.".to_owned()),
            selector: "tab",
            title: "Rename tab".into(),
        };
        let form = rename(
            div().flex().flex_col(),
            &hosts,
            &theme,
            Some(draft()),
            move |button, element| {
                let pressed = pressed.clone();
                element.on_click(move |_, _, _| pressed.set(Some(button)))
            },
        );
        div().size_full().child(
            div()
                .id("form")
                .debug_selector(|| "form".into())
                .child(form),
        )
    }
}

/// A dialog as a workspace action draws it.
struct DialogHarness {
    pressed: Rc<Cell<Option<Button>>>,
    refused: bool,
}

impl Render for DialogHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let pressed = self.pressed.clone();
        let hosts = Dialog {
            buttons: words(),
            error: self.refused.then(|| "that name is taken".to_owned()),
            icon: Some("icons/ui/rename.svg"),
            note: "Names this workspace.".into(),
            target: "agent-launcher".into(),
            title: "Rename workspace".into(),
        };
        div().size_full().child(dialog(
            &hosts,
            &text(),
            &theme,
            danger(),
            Some(draft()),
            move |button, element| {
                let pressed = pressed.clone();
                element.on_click(move |_, _, _| pressed.set(Some(button)))
            },
        ))
    }
}

#[gpui::test]
fn a_rename_form_draws_its_draft_and_reports_the_button_pressed(cx: &mut TestAppContext) {
    let pressed = Rc::new(Cell::new(None));
    let (_, cx) = cx.add_window_view(|_, _| RenameHarness {
        pressed: pressed.clone(),
        refused: true,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        cx.debug_bounds("draft").is_some(),
        "the draft the app built is drawn"
    );
    let cancel = cx.debug_bounds("tab-rename-cancel").unwrap().center();
    cx.simulate_click(cancel, Default::default());
    cx.run_until_parked();
    assert_eq!(pressed.get(), Some(Button::Cancel));
    let submit = cx.debug_bounds("tab-rename-submit").unwrap().center();
    cx.simulate_click(submit, Default::default());
    cx.run_until_parked();
    assert_eq!(pressed.get(), Some(Button::Submit));
}

#[gpui::test]
fn a_form_draws_its_hint_and_its_refusal_only_when_it_has_one(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| RenameHarness {
        pressed: Rc::new(Cell::new(None)),
        refused: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let plain = cx.debug_bounds("form").unwrap().size.height;
    view.update(cx, |view, cx| {
        view.refused = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let refused = cx.debug_bounds("form").unwrap().size.height;
    assert!(
        refused > plain,
        "the hint and the refusal take room: {refused:?} over {plain:?}"
    );
}

#[gpui::test]
fn a_dialog_draws_its_header_note_draft_and_its_refusal(cx: &mut TestAppContext) {
    let pressed = Rc::new(Cell::new(None));
    let (view, cx) = cx.add_window_view(|_, _| DialogHarness {
        pressed: pressed.clone(),
        refused: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    for selector in ["dialog-header", "dialog-title", "dialog-footer", "draft"] {
        assert!(cx.debug_bounds(selector).is_some(), "{selector} is drawn");
    }
    assert!(
        cx.debug_bounds("dialog-error").is_none(),
        "a dialog that was not refused shows no refusal"
    );
    let submit = cx.debug_bounds("dialog-submit").unwrap().center();
    cx.simulate_click(submit, Default::default());
    cx.run_until_parked();
    assert_eq!(pressed.get(), Some(Button::Submit));
    let cancel = cx.debug_bounds("dialog-cancel").unwrap().center();
    cx.simulate_click(cancel, Default::default());
    cx.run_until_parked();
    assert_eq!(pressed.get(), Some(Button::Cancel));
    view.update(cx, |view, cx| {
        view.refused = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(
        cx.debug_bounds("dialog-error").is_some(),
        "the refusal it earned is drawn"
    );
}

/// Which of the form's two ids belongs to which button, so a page that wires
/// one cannot wire the other.
#[test]
fn the_two_buttons_are_named_by_their_page_and_their_decision() {
    let rename = Rename {
        buttons: words(),
        error: None,
        hint: None,
        selector: "pane",
        title: "Rename pane".into(),
    };
    assert_eq!(rename.button(Button::Cancel), "pane-rename-cancel");
    assert_eq!(rename.button(Button::Submit), "pane-rename-submit");
}
