//! Menu bar and bounded, scrollable dropdown presentation: what this window
//! offers the in-window bar for one call, and the handlers and rows it takes
//! back.
// i18n: migrated

use super::OpenMenu;
use crate::SpaceportWindow;
use crate::i18n::fl;
use crate::menu::chrome;
use gpui::prelude::*;
use gpui::*;
use spaceport_menu::bar::{self, Back, Heading, Layer, Row};

impl SpaceportWindow {
    pub(crate) fn render_application_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.application_bar(Layer::Base, window, cx)
    }

    fn application_bar(
        &self,
        layer: Layer,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let headings = self.headings(cx);
        let hosts = self.bar_hosts(window);
        let controls = crate::titlebar::controls(
            self.config.show_window_controls,
            window,
            &self.theme,
            |window, _| {
                window.remove_window();
            },
        );
        let bar = bar::bar(&hosts, layer, &headings, controls, |heading, button| {
            self.interactive_heading(heading, button, cx)
        });
        // Pass controls_enabled so users who hide the WM's title bar can still
        // drag the window from the menu bar.
        crate::titlebar::movable_with_controls(bar, window, self.config.show_window_controls)
    }

    /// The shared headings, installed the first time a window draws the bar.
    fn headings(&self, cx: &mut Context<Self>) -> Vec<(SharedString, bool)> {
        if cx.try_global::<super::super::Headings>().is_none() {
            super::super::install(cx);
        }
        cx.global::<super::super::Headings>().0.clone()
    }

    /// What the in-window bar may read of this window for one call.
    fn bar_hosts(&self, window: &Window) -> bar::Hosts<'_> {
        bar::Hosts {
            compact_label: SharedString::from(fl!("menus-menu")),
            controls_shown: crate::titlebar::show_controls(
                self.config.show_window_controls,
                window,
            ),
            fill: self.fill().panel(self.theme.surface),
            geometry: &self.menu.application_bar,
            label: SharedString::from(fl!("menus-menu-bar")),
            metrics: chrome::metrics(window.viewport_size()),
            open: self.menu.application.as_ref(),
            text: chrome::text(&self.config.ui),
            theme: &self.theme,
        }
    }

    /// One heading's button, wired with what a screen reader, a press, and a
    /// rest on it do.
    fn interactive_heading(
        &self,
        heading: Heading<'_>,
        button: Stateful<Div>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let Heading {
            active,
            compact,
            disabled,
            index,
            name,
            measured,
        } = heading;
        let button = if measured {
            let view = cx.entity().downgrade();
            let button = button.role(Role::MenuItem).aria_label(name.clone());
            let button = crate::accessibility::disabled(button, disabled);
            if disabled {
                button
            } else {
                button
                    .aria_expanded(active)
                    .on_a11y_action(
                        AccessibleAction::Expand,
                        crate::accessibility::update(view.clone(), move |this, window, cx| {
                            this.open_application_menu(index, window, cx)
                        }),
                    )
                    .on_a11y_action(
                        AccessibleAction::Collapse,
                        crate::accessibility::update(view, |this, window, cx| {
                            this.dismiss_menu(window, cx)
                        }),
                    )
            }
        } else {
            button
        };
        button
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                if disabled {
                    return;
                }
                if this
                    .menu
                    .application
                    .as_ref()
                    .is_some_and(|menu| compact || menu.path.first().copied() == index)
                {
                    this.dismiss_menu(window, cx);
                } else {
                    this.open_application_menu(index, window, cx);
                }
            }))
            .on_hover(cx.listener(move |this, hovered, window, cx| {
                if *hovered
                    && !disabled
                    && !compact
                    && this
                        .menu
                        .application
                        .as_ref()
                        .is_some_and(|menu| menu.path.first().copied() != index)
                {
                    this.open_application_menu(index, window, cx);
                }
            }))
    }

    /// A menu item's shortcut in keymap spelling, the way the palette and the
    /// Keyboard Shortcuts page show it. A `cmd-` the user configured is
    /// named for this desktop's platform key, as GPUI's `unparse` names it.
    pub(super) fn application_shortcut(&self, action: &dyn Action, cx: &App) -> String {
        if let Some(command) = action.as_any().downcast_ref::<crate::RunCommand>() {
            let platform = if cfg!(windows) { "win-" } else { "super-" };
            return self
                .keymap()
                .shortcuts(command.command)
                .next()
                .unwrap_or_default()
                .replace("cmd-", platform);
        }
        cx.key_bindings()
            .borrow()
            .bindings_for_action(action)
            .last()
            .map(|binding| {
                binding
                    .keystrokes()
                    .iter()
                    .map(|keystroke| keystroke.unparse(crate::keymap::HOST_OS))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default()
    }

    /// The rows the menu open now shows, each with the words the window says
    /// and the shortcut it runs.
    fn application_rows(&self, menu: &OpenMenu, cx: &App) -> Vec<Row> {
        let items = menu
            .current()
            .map(|menu| menu.items.clone())
            .unwrap_or_else(|| {
                menu.menus
                    .iter()
                    .cloned()
                    .map(OwnedMenuItem::Submenu)
                    .collect()
            });
        items
            .into_iter()
            .map(|item| match item {
                OwnedMenuItem::Separator => Row::Separator,
                OwnedMenuItem::Submenu(menu) => Row::Item {
                    checked: false,
                    detail: SharedString::from("›"),
                    disabled: menu.disabled,
                    name: SharedString::from(menu.name.to_string()),
                },
                OwnedMenuItem::Action {
                    name,
                    action,
                    disabled,
                    checked,
                    ..
                } => Row::Item {
                    checked,
                    detail: SharedString::from(self.application_shortcut(action.as_ref(), cx)),
                    disabled,
                    name: SharedString::from(name),
                },
                OwnedMenuItem::SystemMenu(menu) => Row::Item {
                    checked: false,
                    detail: SharedString::default(),
                    disabled: true,
                    name: SharedString::from(menu.name.to_string()),
                },
            })
            .collect()
    }

    pub(crate) fn render_application_menu(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut overlay = div()
            .id("application-menu-overlay")
            .debug_selector(|| "application-menu-overlay".into())
            .absolute()
            .inset_0()
            .occlude()
            .track_focus(&self.menu.focus)
            .on_key_down(cx.listener(Self::application_key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    cx.stop_propagation();
                    this.dismiss_menu(window, cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, window, cx| {
                    cx.stop_propagation();
                    this.dismiss_menu(window, cx);
                }),
            )
            .on_scroll(|_, _, cx| cx.stop_propagation());
        let Some(menu) = &self.menu.application else {
            return overlay;
        };
        // Focus stays on the overlay while the arrows move the highlight, so
        // the overlay is the menu and the highlighted row its active item.
        overlay = overlay
            .role(Role::Menu)
            .aria_label(menu.current().map_or_else(
                || SharedString::from(fl!("menus-menu")),
                |menu| menu.name.clone(),
            ));
        let hosts = self.bar_hosts(window);
        let rows = self.application_rows(menu, cx);
        let back = menu.current().map(|current| Back {
            label: SharedString::from(fl!("menus-back-to", menu = current.name.to_string())),
            text: SharedString::from(format!("‹  {}", current.name)),
        });
        let dropdown = bar::dropdown(&hosts, rows, back, |index, disabled, row| {
            row.on_hover(cx.listener(move |this, hovered, _, cx| {
                if *hovered
                    && !disabled
                    && let Some(menu) = &mut this.menu.application
                {
                    menu.selected = Some(index);
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.application_choose(index, window, cx);
            }))
        });
        // Repeat the bar above the occluding layer, so switching headings never
        // clicks through the dismissal layer into unrelated window controls.
        // The overlay fills the same content root as the underlying bar. Lay out
        // this copy there too: using cached bounds would feed its stale geometry
        // back into the next frame after a resize or decoration inset change.
        overlay = overlay.child(self.application_bar(Layer::Overlay, window, cx));
        // Moving to another heading keeps the menu still: only one opened from
        // the closed bar drops in.
        overlay.child(bar::place(
            &hosts,
            crate::motion::pop_in(dropdown, "application-menu"),
        ))
    }
}
