// ABOUTME: Shares custom application-menu rows between active and departing panels.
// ABOUTME: Keeps popup focus, actions, and platform cover ownership in the active layer.
//! Menu bar and bounded, scrollable dropdown presentation.
// i18n: migrated

use super::*;
use crate::{fonts::StyledFont, i18n::fl};
use gpui::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BarLayer {
    Base,
    Overlay,
}

impl SheprdWindow {
    pub(super) fn compact_application_bar(&self, window: &Window) -> bool {
        // Reserve space for all three window buttons and their padding before
        // deciding whether the headings fit beside them.
        let controls = if crate::titlebar::show_controls(self.config.show_window_controls, window) {
            112.
        } else {
            0.
        };
        window.viewport_size().width < px(560. * self.config.ui.size / 12. + controls)
    }

    pub(crate) fn render_application_bar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.application_bar(BarLayer::Base, window, cx)
    }

    fn application_bar(
        &self,
        layer: BarLayer,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        if cx.try_global::<super::super::Headings>().is_none() {
            super::super::install(cx);
        }
        let headings = &cx.global::<super::super::Headings>().0;
        let compact = self.compact_application_bar(window);
        let entries: Vec<_> = if compact {
            vec![(None, SharedString::from(fl!("menus-menu")), false)]
        } else {
            headings
                .iter()
                .enumerate()
                .map(|(i, (name, disabled))| (Some(i), name.clone(), *disabled))
                .collect()
        };
        let geometry = self.menu.application_bar.bar.clone();
        let buttons = &self.menu.application_bar.buttons;
        // Only the in-flow bar owns anchor measurements. Its overlay copy
        // must never replace those with geometry from a different frame.
        let measure = layer == BarLayer::Base;
        if measure {
            buttons
                .borrow_mut()
                .resize(entries.len(), Bounds::default());
        }
        let font = &self.config.ui;
        let theme = &self.theme;
        let height = if crate::titlebar::show_controls(self.config.show_window_controls, window) {
            (font.line_height() + 8.).max(crate::titlebar::HEIGHT)
        } else {
            font.line_height() + 8.
        };
        let bar = div()
            .id("application-menu-bar")
            .debug_selector(|| "application-menu-bar".into())
            // The overlay's copy repeats this bar for the pointer only.
            .when(measure, |bar| {
                bar.role(Role::MenuBar).aria_label(fl!("menus-menu-bar"))
            })
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .w_full()
            .h(px(height))
            .px(px(6.))
            // The overlay copy lies on the one in the window, so only that
            // one takes the window's translucency.
            .bg(match layer {
                BarLayer::Base => self.fill().panel(theme.surface),
                BarLayer::Overlay => rgb(theme.surface).into(),
            })
            .border_b_1()
            .border_color(rgb(theme.active))
            .text_font(font)
            .text_size(px(font.size))
            .text_color(rgb(theme.foreground))
            .occlude()
            .on_click(|_, _, cx| cx.stop_propagation())
            .when(measure, |bar| {
                bar.child(
                    canvas(move |bounds, _, _| geometry.set(bounds), |_, _, _, _| {})
                        .absolute()
                        .inset_0(),
                )
            })
            .children(
                entries
                    .into_iter()
                    .enumerate()
                    .map(|(slot, (index, name, disabled))| {
                        let buttons = buttons.clone();
                        let selector = format!("application-menu-{name}");
                        let view = cx.entity().downgrade();
                        let active = self
                            .menu
                            .application
                            .as_ref()
                            .is_some_and(|menu| compact || menu.path.first().copied() == index);
                        div()
                            .id(SharedString::from(selector.clone()))
                            .debug_selector(move || selector.clone())
                            .when(measure, |button| {
                                let heading = button.role(Role::MenuItem).aria_label(name.clone());
                                let heading = crate::accessibility::disabled(heading, disabled);
                                if disabled {
                                    return heading;
                                }
                                heading
                                    .aria_expanded(active)
                                    .on_a11y_action(
                                        AccessibleAction::Expand,
                                        crate::accessibility::update(
                                            view.clone(),
                                            move |this, window, cx| {
                                                this.open_application_menu(index, window, cx)
                                            },
                                        ),
                                    )
                                    .on_a11y_action(
                                        AccessibleAction::Collapse,
                                        crate::accessibility::update(view, |this, window, cx| {
                                            this.dismiss_menu(window, cx)
                                        }),
                                    )
                            })
                            .relative()
                            .flex_none()
                            .h_full()
                            .px(px(10.))
                            // Menu presses must never start a window move or
                            // open the compositor's title-bar context menu.
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                            .flex()
                            .items_center()
                            .rounded(px(crate::config::corners::CONTROL))
                            .when(active, |button| button.bg(rgb(theme.active)))
                            .when(disabled, |button| button.text_color(rgb(theme.muted)))
                            .when(!disabled, |button| {
                                button.cursor_pointer().hover(|s| s.bg(rgb(theme.active)))
                            })
                            .child(name)
                            .when(measure, |button| {
                                button.child(
                                    canvas(
                                        move |bounds, _, _| {
                                            if let Some(button) = buttons.borrow_mut().get_mut(slot)
                                            {
                                                *button = bounds;
                                            }
                                        },
                                        |_, _, _, _| {},
                                    )
                                    .absolute()
                                    .inset_0(),
                                )
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                if disabled {
                                    return;
                                }
                                if this.menu.application.as_ref().is_some_and(|menu| {
                                    compact || menu.path.first().copied() == index
                                }) {
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
                    }),
            )
            .child(div().flex_1().h_full().min_w(px(24.)))
            .children(crate::titlebar::controls(
                self.config.show_window_controls,
                window,
                theme,
                |window, _| {
                    window.remove_window();
                },
            ));
        // Pass controls_enabled so users who hide the WM's title bar can still
        // drag the window from the menu bar.
        crate::titlebar::movable_with_controls(bar, window, self.config.show_window_controls)
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

    pub(crate) fn render_application_menu_layer(
        &self,
        paint: &Paint,
        active: bool,
        motion: (f32, Pixels),
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (shown, lift) = motion;
        let mut overlay = div()
            .id("application-menu-overlay")
            .debug_selector(|| "application-menu-overlay".into())
            .absolute()
            .inset_0()
            .when(active, |overlay| {
                overlay
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
                    .on_scroll(|_, _, cx| cx.stop_propagation())
            });
        // Focus stays on the overlay while the arrows move the highlight, so
        // the overlay is the menu and the highlighted row its active item.
        overlay = overlay.when(active, |overlay| {
            overlay.role(Role::Menu).aria_label(paint.title.clone())
        });
        let bar = paint.bar;
        let content = paint.content;
        let button = paint.button;
        let font = &self.config.ui;
        let theme = &self.theme;
        let width = px(380. * font.size / 12.)
            .min((content.size.width - px(16.)).max(px(0.)))
            .min((window.viewport_size().width - px(16.)).max(px(0.)));
        let top = bar.bottom();
        let items: Vec<_> = paint
            .rows
            .iter()
            .enumerate()
            .map(|(index, item)| {
                self.application_menu_row(index, item, paint.selected == Some(index), active, cx)
                    .into_any_element()
            })
            .collect();
        let rows = div()
            .id("application-menu-rows")
            .debug_selector(|| "application-menu-rows".into())
            .flex()
            .flex_col()
            .min_h_0()
            .when(active, |rows| rows.overflow_y_scroll())
            .when(!active, |rows| rows.overflow_hidden())
            .when_some(
                self.menu.application.as_ref().filter(|_| active),
                |rows, menu| rows.track_scroll(&menu.scroll),
            );
        // Live rows remain direct children: ScrollHandle indexes them for key navigation.
        let rows = if active {
            rows.children(items)
        } else {
            rows.child(
                div()
                    .relative()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .top(paint.offset.y)
                    .children(items),
            )
        };
        let panel = div()
            .id("application-menu-panel")
            .debug_selector(|| "application-menu-panel".into())
            .relative()
            .top(-lift)
            .opacity(shown)
            .w(width)
            .max_h((content.bottom().min(window.viewport_size().height) - top - px(8.)).max(px(0.)))
            .flex()
            .flex_col()
            .overflow_hidden()
            .p(px(5.))
            .bg(rgb(theme.surface))
            .text_color(rgb(theme.foreground))
            .text_font(font)
            .text_size(px(font.size))
            .line_height(px(font.line_height()))
            .border_1()
            .border_color(rgb(theme.active))
            .rounded(px(crate::config::corners::PANEL))
            .shadow_lg()
            .when(active, |panel| {
                panel
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                    .on_click(|_, _, cx| cx.stop_propagation())
            })
            .when_some(paint.back.as_ref(), |panel, name| {
                panel.child(
                    div()
                        .id("application-menu-back")
                        .debug_selector(|| "application-menu-back".into())
                        .flex_none()
                        .p(px(8.))
                        .child(format!("‹  {name}"))
                        .when(active, |back| {
                            back.role(Role::MenuItem)
                                .aria_label(fl!("menus-back-to", menu = name.to_string()))
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.application_back(cx);
                                }))
                        }),
                )
            })
            .child(rows);
        // Only the active panel covers native Windows pages.
        #[cfg(any(windows, test, feature = "integration-test"))]
        let panel = panel.when(active, |panel| {
            let cover = self.menu.cover.clone();
            panel.child(
                canvas(
                    move |bounds, _, _| cover.set(crate::menu::Cover::Panel(bounds.dilate(px(8.)))),
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
        });
        // Only the active popup repeats headings above its dismissal layer.
        overlay = overlay.when(active, |overlay| {
            overlay.child(self.application_bar(BarLayer::Overlay, window, cx))
        });
        overlay.child(
            anchored()
                .position(point(
                    button
                        .left()
                        .min((content.right() - width - px(8.)).max(content.left() + px(8.)))
                        .max(content.left() + px(8.)),
                    top,
                ))
                .snap_to_window_with_margin(Edges::all(px(8.)))
                .child(panel),
        )
    }

    fn application_menu_row(
        &self,
        index: usize,
        row: &paint::Row,
        selected: bool,
        active: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let font = &self.config.ui;
        let theme = &self.theme;
        let (name, disabled, checked, detail) = match row {
            paint::Row::Separator => {
                return div()
                    .id(("application-separator", index))
                    .flex_none()
                    .h(px(1.))
                    .my(px(4.))
                    .mx(px(8.))
                    .bg(rgb(theme.active));
            }
            paint::Row::Item {
                name,
                disabled,
                checked,
                detail,
            } => (name.clone(), *disabled, *checked, detail.clone()),
        };
        let selector = format!("application-item-{name}");
        let spoken = match detail.as_ref() {
            "" | "›" => name.to_string(),
            shortcut => format!("{name}, {shortcut}"),
        };
        let item = div().id(("application-item", index)).when(active, |item| {
            item.role(if checked {
                Role::MenuItemCheckBox
            } else {
                Role::MenuItem
            })
            .aria_label(spoken)
            .when(checked, |item| item.aria_toggled(Toggled::True))
            .when(selected, |item| item.aria_active_descendant())
        });
        item.when(active, |item| {
            crate::accessibility::disabled(item, disabled)
        })
        .debug_selector(move || selector.clone())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .px(px(8.))
        .min_h(px(font.line_height() + 10.))
        .rounded(px(crate::config::corners::CONTROL))
        .when(selected, |row| row.bg(rgb(theme.active)))
        .when(disabled, |row| row.text_color(rgb(theme.muted)))
        .when(active && !disabled, |row| row.cursor_pointer())
        .child(
            div()
                .flex_none()
                .w(px(14.))
                .child(if checked { "✓" } else { "" }),
        )
        .child(div().flex_1().min_w_0().overflow_hidden().child(name))
        .child(div().flex_none().text_color(rgb(theme.muted)).child(detail))
        .when(active, |row| {
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
        })
    }
}
