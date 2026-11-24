//! Linux and Windows presentation of the same menu tree installed in macOS's menu bar.
//! The shared popup state isolates terminal input; actions are dispatched only
//! after restoring the focus from before the menu opened.

#[cfg(all(feature = "integration-test", target_os = "linux"))]
pub(crate) mod native;
mod render;

use crate::{SpaceportWindow, menu::Page};
use gpui::*;
pub(crate) use spaceport_menu::bar::{Geometry, OpenMenu};

/// Action availability belongs to the previously focused element, not the
/// popup that is about to take focus. OS-only menu entries aren't selectable.
fn validate(menu: &mut OwnedMenu, window: &Window, cx: &mut App) {
    for item in &mut menu.items {
        match item {
            OwnedMenuItem::Action {
                action, disabled, ..
            } => {
                // The window's dispatch tree misses app-wide handlers such as
                // Quit's, which only the app's own check counts.
                *disabled |= !window.is_action_available(action.as_ref(), cx)
                    && !cx.is_action_available(action.as_ref());
            }
            OwnedMenuItem::Submenu(menu) => validate(menu, window, cx),
            _ => {}
        }
    }
}

impl SpaceportWindow {
    pub(crate) fn application_menu_shortcut(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if super::IN_WINDOW
            && self.menu.page.is_none()
            && event.keystroke.key == "f10"
            && event.keystroke.modifiers == Modifiers::default()
        {
            self.open_application_menu(Some(0), window, cx);
            cx.stop_propagation();
            window.prevent_default();
        }
    }

    pub(crate) fn open_application_menu(
        &mut self,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(menu) = &mut self.menu.application {
            if index.is_some_and(|index| menu.menus.get(index).is_none_or(|menu| menu.disabled)) {
                return;
            }
            menu.path = index.into_iter().collect();
            menu.select_first();
            cx.notify();
            return;
        }
        let return_focus = window.focused(cx);
        let mut menus: Vec<_> = super::menus(&self.config)
            .into_iter()
            .map(Menu::owned)
            .collect();
        if index.is_some_and(|index| menus.get(index).is_none_or(|menu| menu.disabled)) {
            return;
        }
        for menu in &mut menus {
            validate(menu, window, cx);
        }
        if !self.open_menu(window, cx) {
            return;
        }
        let mut menu = OpenMenu {
            menus,
            path: index.into_iter().collect(),
            selected: None,
            scroll: ScrollHandle::new(),
            return_focus,
        };
        menu.select_first();
        self.menu.application = Some(menu);
        self.menu.page = Some(Page::Application);
        cx.notify();
    }

    /// Closes the open popup, which keeps its own copy of the menus, once a
    /// config change rebuilds the bar so it cannot offer items that left.
    pub(crate) fn close_stale_application_menu(&mut self, cx: &mut Context<Self>) {
        if self.menu.application.take().is_none() {
            return;
        }
        self.menu.page = None;
        cx.notify();
    }

    fn application_back(&mut self, cx: &mut Context<Self>) {
        if let Some(menu) = &mut self.menu.application {
            let selected = menu.path.pop();
            menu.select_first();
            menu.selected = selected;
            if let Some(index) = selected {
                menu.scroll.scroll_to_item(index);
            }
            cx.notify();
        }
    }

    fn application_choose(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &mut self.menu.application else {
            return;
        };
        if !menu.selectable().contains(&index) {
            return;
        }
        let action = menu
            .current()
            .and_then(|menu| menu.items.get(index))
            .and_then(|item| {
                if let OwnedMenuItem::Action { action, .. } = item {
                    Some(action.boxed_clone())
                } else {
                    None
                }
            });
        if let Some(action) = action {
            self.dismiss_menu(window, cx);
            // GPUI dispatches on the next frame, after the popup's focus tree
            // has been replaced by the original input destination.
            window.dispatch_action(action, cx);
        } else {
            menu.path.push(index);
            menu.select_first();
            cx.notify();
        }
    }

    fn application_cycle(&mut self, backwards: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(menu) = &self.menu.application else {
            return;
        };
        let indexes: Vec<_> = menu
            .menus
            .iter()
            .enumerate()
            .filter_map(|(i, menu)| (!menu.disabled).then_some(i))
            .collect();
        if indexes.is_empty() {
            return;
        }
        let current = menu
            .path
            .first()
            .and_then(|i| indexes.iter().position(|index| i == index))
            .unwrap_or(0);
        let next = if backwards {
            (current + indexes.len() - 1) % indexes.len()
        } else {
            (current + 1) % indexes.len()
        };
        self.open_application_menu(Some(indexes[next]), window, cx);
    }

    fn application_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        window.prevent_default();
        let Some(menu) = &mut self.menu.application else {
            return;
        };
        match event.keystroke.key.as_str() {
            "escape" | "f10" => self.dismiss_menu(window, cx),
            key @ ("up" | "down" | "home" | "end") => {
                menu.step(key);
                cx.notify();
            }
            "enter" | "space" => {
                if let Some(index) = menu.selected {
                    self.application_choose(index, window, cx);
                }
            }
            "left" if menu.path.len() > 1 => self.application_back(cx),
            "left" => self.application_cycle(true, window, cx),
            "right" => {
                let submenu = menu.current().is_none()
                    || menu.selected.is_some_and(|index| {
                        matches!(
                            menu.current().and_then(|menu| menu.items.get(index)),
                            Some(OwnedMenuItem::Submenu(_))
                        )
                    });
                if submenu {
                    if let Some(index) = menu.selected {
                        self.application_choose(index, window, cx);
                    }
                } else {
                    self.application_cycle(false, window, cx);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
