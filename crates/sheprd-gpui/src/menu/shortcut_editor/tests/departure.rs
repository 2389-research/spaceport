// ABOUTME: Exercises frozen shortcut controls without a parent input shield.
// ABOUTME: Uses an invalid replacement draft so a broken save guard cannot write personal settings.
use super::*;
use crate::{menu::settings::KeybindsPaint, sidebar::layout_tests::fixture_window};
use gpui::{Entity, Render, VisualTestContext};

struct Frozen {
    source: Entity<SheprdWindow>,
    paint: KeybindsPaint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(self.source.update(cx, |source, cx| {
                source.render_keybinds_paint(&self.paint, cx)
            }))
    }
}

fn preview(cx: &mut TestAppContext) -> (Entity<Frozen>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        let paint = source.update(cx, |source, cx| {
            source.open_keybinds(window, cx);
            source
                .menu
                .keybinds_search
                .as_ref()
                .unwrap()
                .update(cx, |input, cx| {
                    input.set_text_selected("New Tab", cx);
                });
            source.edit_shortcut(info(Command::Tab), window, cx);
            let paint = source.keybinds_paint(cx);
            source.dismiss_menu(window, cx);
            source.open_keybinds(window, cx);
            source.edit_shortcut(info(Command::Workspace), window, cx);
            source
                .menu
                .shortcut_editor
                .as_ref()
                .unwrap()
                .input
                .update(cx, |input, cx| {
                    input.set_text_selected("t", cx);
                });
            paint
        });
        Frozen { source, paint }
    })
}

#[gpui::test]
fn frozen_controls_cannot_change_or_save_a_replacement_editor(cx: &mut TestAppContext) {
    for selector in [
        "keybinds-close",
        "edit-shortcut-new_tab",
        "shortcut-cancel",
        "shortcut-save",
    ] {
        let (view, cx) = preview(cx);
        let original = view.read_with(cx, |view, cx| {
            view.source
                .read(cx)
                .menu
                .shortcut_editor
                .as_ref()
                .unwrap()
                .input
                .entity_id()
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let button = cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("{selector} is painted"));
        cx.simulate_click(button.center(), Modifiers::default());
        view.read_with(cx, |view, cx| {
            let source = view.source.read(cx);
            assert_eq!(source.menu.page, Some(Page::Keybinds), "{selector}");
            let editor = source.menu.shortcut_editor.as_ref().unwrap();
            assert_eq!(editor.info.command, Command::Workspace, "{selector}");
            assert_eq!(editor.input.entity_id(), original, "{selector}");
            assert_eq!(editor.input.read(cx).text(), "t", "{selector}");
            assert!(
                editor.error.is_none(),
                "{selector} must not validate the new draft"
            );
            assert!(!editor.saving, "{selector} must not start a save");
        });
    }
}
