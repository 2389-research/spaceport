// ABOUTME: Activates the real sidebar spacing reset through keyboard focus.
// ABOUTME: Reuses the Settings preference writer fixture for exact saved edits.
#![allow(clippy::unwrap_used)]

use super::*;
use crate::settings_window::{
    controls::preferences::PreferenceIo,
    tests::{press_key, skill_fixture},
};
use core::prelude::v1::test;
use std::sync::{Arc, Mutex};

struct Reset(Entity<SettingsWindow>, FocusHandle);
impl Render for Reset {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().track_focus(&self.1).child(
            self.0
                .update(cx, |view, cx| view.spacing_reset(Spacing::Indent, true, cx)),
        )
    }
}

#[gpui::test]
fn a_spacing_override_can_be_reset_without_a_pointer(cx: &mut TestAppContext) {
    let edits = Arc::new(Mutex::new(Vec::new()));
    let written = edits.clone();
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = cx.new(|cx| {
            let mut view = skill_fixture(window, cx);
            view.config.sidebar_style.overrides.indent = Some(30.);
            view.loaded_config.sidebar_style.overrides.indent = Some(30.);
            view.controls.preference_io = Some(PreferenceIo::recording(move |edit| {
                written.lock().unwrap().push(edit);
            }));
            view
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Reset(view, focus)
    });
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        window.focus_next(cx);
        window.draw(cx).clear(cx);
    });
    press_key(cx, "enter");
    cx.run_until_parked();
    assert_eq!(*edits.lock().unwrap(), [Preference::SidebarIndent(None)]);
}
