// ABOUTME: Paints the window from prepared state and bounded caches.
// ABOUTME: Keeps accessibility subtrees present while preserving ordinary render reuse.
//! Painting the window from prepared state. Render reads bounded caches and
//! the latest projection only: it never queries the daemon, touches disk, or
//! starts a process.

use super::{SpaceportWindow, tab_strip::Reach};
use crate::direction::Logical;
use crate::{
    Minimize, PlaySound, RunCommand, ZoomWindow,
    actions::{RingBellPreview, ShowToastPreview},
    browser::{Shown, Slot},
    fonts::StyledFont,
    terminal::*,
    terminal_painter::{self, Ground},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::PaneSurfaceFrame;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

mod cells;

impl SpaceportWindow {
    /// Config warnings, then the daemon's announcement, stacked over the
    /// top-right of the terminal area below the tab strip, whose buttons stay
    /// reachable. A menu page owns the window's attention, and zen mode keeps
    /// the window clear; they wait behind either.
    fn render_notices(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.notices_allowed() {
            return None;
        }
        let mut cards = self.config_diagnostic_cards(cx);
        cards.extend(self.announcement_card(cx));
        if cards.is_empty() {
            return None;
        }
        // Spans the terminal area so a narrow window shrinks the cards rather
        // than pushing them off the left edge. Each card sits in its own row:
        // a column only shrinks its children along its height, so a card with
        // a long line kept its full width there. The stack ends above the
        // window's foot, and a card that scrolls gives up height so the cards
        // under it stay on screen; the rest keep theirs. Neither the stack nor
        // a row takes the pointer, so the terminal around the cards scrolls
        // and selects as before.
        Some(
            div()
                .absolute()
                .top(px(self.tab_strip_height() + 8.))
                .bottom(px(8.))
                .left(px(8.))
                .right(px(8.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(cards.into_iter().map(|Notice { card, shrinks }| {
                    div()
                        .flex()
                        .justify_end()
                        .map(|row| match shrinks {
                            true => row.min_h_0(),
                            false => row.flex_none(),
                        })
                        .child(card)
                }))
                .into_any_element(),
        )
    }
}

/// One card in the notice stack, and whether it scrolls, so it can give up
/// height to keep the stack inside the window.
pub(super) struct Notice {
    pub(super) card: AnyElement,
    pub(super) shrinks: bool,
}

impl SpaceportWindow {
    fn notices_allowed(&self) -> bool {
        self.menu.page.is_none() && !self.zen_active()
    }

    /// Whether [`Self::render_notices`] draws a card, so the keyboard can
    /// reach the cards as a region of their own.
    pub(super) fn notices_shown(&self) -> bool {
        self.notices_allowed()
            && (self.gui_config_diagnostic.visible().is_some()
                || self.icon_font_notice.visible().is_some()
                || self
                    .endpoints
                    .get(self.selected_endpoint)
                    .is_some_and(|endpoint| endpoint.config_diagnostic.visible().is_some())
                || self.live.product_announcement().is_some())
    }
}

impl Render for SpaceportWindow {
    /// A frame in three steps: the chrome that moves itself forward, the
    /// state it draws from, and the body and root built from that state. Only
    /// the first step changes anything; the rest read what it left.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.tick_frame(window, cx);
        let (paint, areas) = self.prepare_frame(window, cx);
        let terminal = self.terminal_area(paint, window, cx);
        let body = self.render_body(areas, terminal, window, cx);
        self.render_root(body, window, cx)
    }
}

impl SpaceportWindow {
    /// Everything that moves on its own this frame: focus that left a view,
    /// chrome that slides, and the themes and glyphs that follow it.
    fn tick_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.restore_menu_focus(window, cx);
        self.leave_vanished_region(window, cx);
        self.sync_backdrop(window, cx);
        self.tick_chrome(window, cx);
        self.tick_menu_appear(window);
        self.tick_sidebar_slide(window, cx);
        if self
            .chats
            .glide(crate::motion::now(), self.config.chat.glide)
        {
            window.request_animation_frame();
        }
        self.sync_device_themes(cx);
    }

    /// What this frame draws from, measured once: the cells to paint, the
    /// highlights they carry, the boxes they sit in, and the groups they
    /// belong to. Nothing here asks the daemon or the disk.
    fn prepare_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) -> (Paint, Areas) {
        let (mut paint, mut areas) = self.prepare_paint(window, cx);
        paint.find_bar = self.render_find_bar(paint.surface.as_deref(), cx);
        paint.copy_badge = self.render_copy_mode_badge(paint.surface.as_deref(), cx);
        paint.chats = self.render_chats(paint.surface.as_deref(), cx);
        self.prepare_groups(&mut areas, window, cx);
        (paint, areas)
    }

    /// What the cells paint and the boxes they sit in: the frame on screen,
    /// the highlights over it, and the cursors that follow the pointer.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 123 lines")]
    fn prepare_paint(&mut self, window: &mut Window, cx: &mut Context<Self>) -> (Paint, Areas) {
        let fill = self.fill();
        let ground = self.ground();
        self.viewport_width = f32::from(window.viewport_size().width);
        let font = crate::config::font_for(&self.config.terminal);
        // Parked groups paint with the same face as the window's terminal.
        let parked_font = font.clone();
        let cell_height = self.config.terminal.line_height();
        self.painter.borrow_mut().set_appearance(
            self.config.terminal.size,
            cell_height,
            self.terminal_theme().clone(),
        );
        self.cell_width = self.painter.borrow_mut().cell_width(&font, window, cx);
        // Registers the window for surface-only redraws; see `redraw_terminal`.
        self.surface_signal.read(cx);
        // Paints the frame on screen, which during a focus change is the one
        // presented before it: the terminal area never blanks between two
        // projections. What the client knows to be current stays in `live`.
        let picture = self.presentation.picture(&self.live);
        let surface = picture.as_ref().map(|picture| picture.frame.clone());
        let images = picture.as_ref().map(|picture| picture.images.clone());
        // A group picking the tab another shows paints this same frame.
        let window_frame = picture;
        let entity = cx.entity();
        let focus = self.focus.clone();
        let menu_open = self.menu.page.is_some();
        let cell_width = self.cell_width;
        let metrics = self.metrics();
        let painter = self.painter.clone();
        // The highlight belongs to the frame that owns the cells, so only one
        // of the panes and the popup paints it.
        let popup_highlights = surface
            .as_deref()
            .and_then(|surface| surface.popup.as_ref())
            .map(|popup| {
                self.selection_highlights(
                    surface.as_deref(),
                    cell_width,
                    cell_height,
                    self.selection
                        .as_ref()
                        .is_some_and(|selection| selection.in_popup(&popup.terminal_id)),
                )
            })
            .unwrap_or_default();
        let pane_selection = self.selection_highlights(
            surface.as_deref(),
            cell_width,
            cell_height,
            self.selection
                .as_ref()
                .is_some_and(|selection| selection.in_panes()),
        );
        let link_rows = self.hovered_link_rows(surface.as_deref());
        // Search matches, mapped onto the frame on screen, tint below the
        // selection, which reads as chosen over them. A popup covers the
        // panes, so their matches stay under it.
        let mut highlights = surface
            .as_deref()
            .map(|surface| {
                let mut highlights = self.find_highlights(surface);
                highlights.extend(self.copy_mode_highlights(surface));
                highlights
            })
            .unwrap_or_default();
        highlights.extend(pane_selection);
        let look = super::regions::Look {
            font: font.clone(),
            font_size: self.config.terminal.size,
            metrics,
            theme: self.terminal_theme().clone(),
            ground,
        };
        if !self.slides_allowed() {
            self.presentation.scroll.clear();
        }
        let now = crate::motion::now();
        let slide = self.presentation.scroll.slide(now);
        // A link in a pane drawn mid-row underlines where its text is drawn.
        let link_shift = link_rows
            .first()
            .and_then(|(row, columns)| self.shift_at(metrics.origin(columns.start, *row)));
        if self.presentation.scroll.moving(now) {
            // Advance the drawing on the next refresh through the surface
            // signal: notifying this view would also rebuild the cached sidebar.
            let signal = self.surface_signal.clone();
            window.on_next_frame(move |_, cx| signal.update(cx, |_, cx| cx.notify()));
        }
        let regions = self.terminal_regions(surface.as_ref(), &highlights, slide, &look, cx);
        // Without regions the canvas paints the whole grid, images included.
        let whole = regions.is_empty();
        // The IME composition paints inline at the input cursor; a menu's
        // text field shows its own.
        // It anchors to the live surface, as the IME's candidate window does,
        // so a retained frame never separates the text from the window.
        let marked = (!menu_open && !self.marked.is_empty()).then(|| {
            (
                self.marked.clone(),
                self.live.surface.clone(),
                self.ime_shift(),
            )
        });
        self.hovered_terminal_link =
            self.terminal_link_hovered(window.mouse_position(), window.modifiers());
        self.split_cursor = self.split_cursor_at(window.mouse_position());
        // Pad the terminal itself: the canvas bounds that painting, hit testing,
        // and IME placement all read then already exclude the padding. Only a
        // group that draws a terminal or stands in for one takes it: every
        // other kind of tab pads its own content.
        let padding = self.config.terminal_padding;
        let paint = Paint {
            cell_height,
            cell_width,
            chats: Vec::new(),
            copy_badge: None,
            entity,
            fill,
            find_bar: None,
            focus,
            font,
            ground,
            highlights,
            images,
            link_rows,
            link_shift,
            marked,
            menu_open,
            metrics,
            padding,
            painter,
            popup_highlights,
            regions,
            surface,
            whole,
        };
        let areas = Areas {
            cell_height,
            fill,
            parked_font,
            shown: Vec::new(),
            sidebar: None,
            slots: Vec::new(),
            terminal_slot: None,
            window_frame,
        };
        (paint, areas)
    }

    /// The sidebar beside the groups, the groups this frame lays out, and
    /// where each shows its tab.
    fn prepare_groups(&mut self, areas: &mut Areas, window: &mut Window, cx: &mut Context<Self>) {
        self.ensure_layout();
        self.forget_gone_strips();
        let slots = self.drawn_group_slots();
        let shown: Vec<Shown> = slots
            .iter()
            .map(|slot| self.group_shown(slot.id, cx))
            .collect();
        // The window's own terminal, with its input, goes to the group holding
        // its connection; other terminal groups paint parked connections.
        let primary = self.primary_group();
        let terminal_slot = slots
            .iter()
            .zip(&shown)
            .find(|(slot, shown)| {
                **shown == Shown::Terminal && primary.is_none_or(|group| group == slot.id)
            })
            .map(|(slot, _)| *slot);
        let sidebar = self
            .sidebar_column_width(f32::from(window.viewport_size().width))
            .map(|width| {
                (
                    crate::sidebar::view_element(
                        &self.sidebar_view,
                        width,
                        window.is_a11y_active(),
                    ),
                    width,
                )
            });
        // The tab strip reserves its moving marks afresh; a hidden sidebar
        // leaves none behind.
        if let Some(marks) = crate::mark_layer::Marks::of(window, cx) {
            marks.begin_frame(cx.entity_id());
            if sidebar.is_none() {
                marks.forget(self.sidebar_view.entity_id());
            }
        }
        areas.sidebar = sidebar;
        areas.slots = slots;
        areas.shown = shown;
        areas.terminal_slot = terminal_slot;
    }

    /// The rows the pointer's link underlines. Like the highlight, the
    /// underline paints with the frame that owns its cells, and only while
    /// that frame shows the content the daemon resolved it from. Without one,
    /// the local reading under the pointer is underlined instead, on every row
    /// it covers.
    fn hovered_link_rows(
        &self,
        surface: Option<&PaneSurfaceFrame>,
    ) -> Vec<(u16, std::ops::Range<u16>)> {
        surface
            .zip(self.hovered_daemon_link())
            .filter(|(surface, link)| link.cell.current(surface))
            .map(|(_, link)| link.frame_rows().collect())
            .or_else(|| Some(self.hovered_local_link()?.rows().collect()))
            .unwrap_or_default()
    }

    /// The selection's rows as grid highlights, which paint with the frame
    /// that owns the cells rather than being recomputed from the pointer.
    fn selection_highlights(
        &self,
        surface: Option<&PaneSurfaceFrame>,
        cell_width: f32,
        cell_height: f32,
        owned: bool,
    ) -> Vec<terminal_painter::Highlight> {
        surface
            .zip(self.selection.as_ref().filter(|_| owned))
            .into_iter()
            .flat_map(|(surface, selection)| selection.rows(surface, cell_width, cell_height))
            .map(|(row, columns)| terminal_painter::Highlight {
                row,
                columns,
                tint: terminal_painter::Tint::Selection,
            })
            .collect()
    }

    /// The terminal's box: the cells, and the input that reaches them.
    fn terminal_area(
        &mut self,
        paint: Paint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let fill = paint.fill;
        let padding = paint.padding;
        crate::config::pad(div(), padding)
            .id("terminal")
            .debug_selector(|| "terminal".into())
            .when(self.hovered_terminal_link, |terminal| {
                terminal.cursor_pointer()
            })
            .when_some(self.split_cursor, |terminal, cursor| {
                terminal.cursor(cursor)
            })
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                this.pane_pointer_moved(event, window, cx);
            }))
            // Holding the link modifier over a link in a mouse-reporting
            // application changes what a click does, so the pointer follows.
            .on_modifiers_changed(
                cx.listener(|this, event: &ModifiersChangedEvent, window, cx| {
                    this.hover_link(window.mouse_position(), event.modifiers, cx);
                    let hovered =
                        this.terminal_link_hovered(window.mouse_position(), event.modifiers);
                    if hovered != this.hovered_terminal_link {
                        this.hovered_terminal_link = hovered;
                        cx.notify();
                    }
                }),
            )
            .on_click(cx.listener(Self::open_terminal_link))
            .relative()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .bg(fill.base(self.terminal_theme().background))
            .map(|terminal| self.accessible_terminal(terminal, window, cx))
            .capture_key_down(cx.listener(Self::chat_zoom_key))
            .on_key_down(cx.listener(Self::key_down))
            .on_key_up(cx.listener(Self::key_up))
            // The terminal has nothing for Cut to act on, and Copy only while
            // a released selection is still highlighted.
            .on_action(cx.listener(|this, _: &crate::actions::Paste, _, cx| this.paste(cx)))
            .on_action(cx.listener(|this, _: &crate::actions::SelectAll, _, cx| {
                this.select_all(cx);
            }))
            .when(self.selection_retained(), |terminal| {
                terminal.on_action(cx.listener(|this, _: &crate::actions::Copy, _, cx| {
                    this.copy_retained_selection(cx);
                }))
            })
            .on_scroll(cx.listener(Self::scroll_wheel))
            .on_drop(cx.listener(Self::drop_terminal_files))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.terminal_mouse_down(event, window, cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    if this.terminal_mouse_down(event, window, cx) {
                        return;
                    }
                    cx.stop_propagation();
                    this.open_pane_menu_at(event.position, window, cx);
                    this.menu.opening_right_click = this.menu.page == Some(crate::menu::Page::Pane);
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.note_input();
                    if this.scrollbar_mouse_down(event, cx)
                        || this.split_mouse_down(event, cx)
                        || this.terminal_mouse_down(event, window, cx)
                    {
                        return;
                    }
                    this.pressed_terminal_link =
                        this.terminal_link_press(event.position, event.modifiers);
                    if this.menu.page.is_some() {
                        return;
                    }
                    // A press on a link may still turn into a drag across it,
                    // so the selection starts either way; the click that opens
                    // the link is the one that never left its half-cell.
                    this.begin_selection(event.position, event.click_count, cx);
                    if this.pressed_terminal_link.is_some() {
                        cx.stop_propagation();
                        return;
                    }
                    window.focus(&this.focus, cx);
                    if this.input_ready()
                        && let Some(surface) = &this.live.surface
                    {
                        let pane = pane_at(surface, this.bounds, event.position, this.metrics())
                            .map(str::to_owned);
                        if let Some(id) = pane {
                            this.focus_clicked_pane(&id, cx);
                        }
                    }
                }),
            )
            // The cached regions paint the cells; the canvas above them takes
            // input and paints what changes without them. Both fill this box,
            // so every region shares the canvas's bounds. What is laid over
            // the cells sits in the same box, inside the terminal's padding.
            // A frame the daemon has not resized yet may be larger than the
            // box, so the box clips it and the padding stays clear.
            .child(self.terminal_cells(paint, window, cx))
    }

    /// The terminal, every group's body behind it, and the content they make.
    fn render_body(
        &mut self,
        areas: Areas,
        terminal: Stateful<Div>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Body {
        let fill = areas.fill;
        let Areas {
            cell_height,
            parked_font,
            shown,
            sidebar,
            slots,
            terminal_slot,
            window_frame,
            ..
        } = areas;
        // A focus handle belongs to one element: the terminal when drawn,
        // otherwise the group in use.
        let keyboard = terminal_slot
            .is_none()
            .then(|| self.active_group())
            .flatten();
        let mut terminal = Some(terminal);
        // An activity bar section takes the body; its title bar is the
        // window's own, with no group's tab strip in it.
        let section = self.render_activity_section(cx);
        let merged = section.is_none()
            && slots
                .first()
                .is_some_and(|first| self.tabs_in_titlebar(first.id, cx));
        let count = slots.len();
        let mut groups = Vec::with_capacity(count);
        // The pictures the last frame painted. Each group keeps its own view
        // out of this while what it paints has not changed; whatever no group
        // keeps is dropped with the views it painted.
        let mut previous = std::mem::take(&mut self.group_pictures);
        for (slot, shown) in slots.into_iter().zip(shown) {
            let owns_keyboard = keyboard == Some(slot.id);
            let tab = match &shown {
                Shown::Page(id) => cx
                    .try_global::<crate::browser::Store>()
                    .and_then(|store| store.get(*id))
                    .cloned(),
                _ => None,
            };
            let body = match (&shown, tab) {
                // No Herdr 1 is installed: herdr-core's shell stands in.
                (Shown::Terminal, _)
                    if terminal_slot == Some(slot) && self.core_fallback_view().is_some() =>
                {
                    self.core_fallback_view()
                        .map(IntoElement::into_any_element)
                        .unwrap_or_else(|| div().into_any_element())
                }
                (Shown::Terminal, _) if terminal_slot == Some(slot) => terminal
                    .take()
                    .map(IntoElement::into_any_element)
                    .unwrap_or_else(|| div().into_any_element()),
                (Shown::Terminal, _) if self.shows_parked_terminal(slot.id, cx) => self
                    .render_parked_terminal(
                        &mut previous,
                        slot,
                        parked_font.clone(),
                        cell_height,
                        cx,
                    ),
                (Shown::Page(_), Some(tab)) => match tab.location {
                    None if tab.core_group().is_some() => {
                        self.render_core_terminal_tab(slot, &tab, cx)
                    }
                    Some(crate::browser::Location::Devices) => {
                        self.render_devices_tab(slot, &tab, cx)
                    }
                    Some(crate::browser::Location::CoreTerminal { .. }) => {
                        self.render_core_terminal_tab(slot, &tab, cx)
                    }
                    // A blank tab outside a Core group shows nothing.
                    None => div().into_any_element(),
                },
                (Shown::Elsewhere(tab), _) => match self.live_frame_of(tab, window_frame.clone()) {
                    Some(frame) => self.render_terminal_mirror(
                        &mut previous,
                        slot,
                        Some(frame),
                        parked_font.clone(),
                        cell_height,
                        cx,
                    ),
                    None => self.render_stand_in(slot, &shown, owns_keyboard, cx),
                },
                _ => self.render_stand_in(slot, &shown, owns_keyboard, cx),
            };
            let ends = merged.then(|| crate::titlebar::Ends::of(slot.index, count));
            let reach = Reach::of(
                if self.folding_groups().is_empty() {
                    count
                } else {
                    0
                },
                slot.index,
            );
            groups.push(self.render_group(slot, body, ends, reach, window, cx));
        }
        let content = self.render_groups(groups, window, cx);
        let content = section.unwrap_or_else(|| self.zen_column(content));
        Body {
            content,
            fill,
            merged,
            sidebar,
        }
    }

    /// The window's root: the actions it answers, the bars it stacks, and the
    /// frame around them.
    fn render_root(
        &mut self,
        body: Body,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.groups_moving() | self.tabs_growing() | self.chrome_sliding() | self.flash_moving()
        {
            window.request_animation_frame();
        }
        // Zen mode keeps toasts and menus: they carry errors and what the
        // user asked for, not chrome.
        let zen = self.zen_layout();
        let root = div()
            .capture_key_down(cx.listener(Self::application_menu_shortcut))
            .capture_key_down(cx.listener(Self::chrome_focus_key))
            .on_modifiers_changed(cx.listener(Self::double_shift_modifiers))
            .capture_any_mouse_down(cx.listener(|this, _, _, _| this.shift_taps.cancel()))
            // Shift-taps are cancelled wherever the pointer goes in the
            // window, which an element of its own sees before any row does.
            .child({
                let entity = cx.weak_entity();
                let menu_bounds = self.menu.application_bar.content.clone();
                canvas(
                    move |bounds, _, _| menu_bounds.set(bounds),
                    move |_, _, window, _| {
                        let scroll_entity = entity.clone();
                        window.on_mouse_event(move |_: &ScrollEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture {
                                let _ =
                                    scroll_entity.update(cx, |this, _| this.shift_taps.cancel());
                            }
                        });
                        let entity = entity.clone();
                        window.on_mouse_event(move |_: &MouseMoveEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture {
                                let _ = entity.update(cx, |this, _| this.shift_taps.cancel());
                            }
                        });
                    },
                )
                .absolute()
                .size_full()
            });
        let root = self.root_events(root, cx);
        let root = self.root_body(root, body, zen, window, cx);
        crate::titlebar::frame(window, self.theme.active, root)
    }

    /// What the window answers wherever the keyboard is: the actions it runs
    /// and the text it sets.
    fn root_events(&self, root: Div, cx: &mut Context<Self>) -> Div {
        root.on_action(
            cx.listener(|this, action: &crate::actions::SetLayout, _, cx| {
                this.set_layout(action.mode, cx);
            }),
        )
        .on_action(
            cx.listener(|this, action: &crate::actions::SetChatLayout, _, cx| {
                this.set_chat_layout(action.mode, cx);
            }),
        )
        .on_action(
            cx.listener(|this, action: &crate::actions::SetTabLayout, _, cx| {
                this.set_tab_layout(action.mode, cx);
            }),
        )
        .on_action(cx.listener(|this, action: &RunCommand, window, cx| {
            this.command(action.command, window, cx);
        }))
        .on_action(cx.listener(|_, _: &Minimize, window, _| {
            window.minimize_window();
        }))
        .on_action(cx.listener(|_, _: &ZoomWindow, window, _| {
            window.zoom_window();
        }))
        .on_action(cx.listener(|this, action: &ShowToastPreview, _, cx| {
            this.show_toast_preview(action.kind, cx);
        }))
        .on_action(cx.listener(
            |this, _: &crate::actions::ShowSystemNotificationPreview, window, cx| {
                this.show_system_notification_preview(window, cx);
            },
        ))
        .on_action(cx.listener(|this, _: &PlaySound, _, _| {
            this.sound.preview();
        }))
        .on_action(cx.listener(|this, _: &RingBellPreview, window, cx| {
            this.preview_bell(window, cx);
        }))
        // A mouse's side buttons walk the focus trail, as they do pages
        // in a browser or folders in a file manager.
        .on_mouse_down(
            MouseButton::Navigate(NavigationDirection::Back),
            cx.listener(|this, _, window, cx| {
                this.command(crate::controls::Command::Back, window, cx);
            }),
        )
        .on_mouse_down(
            MouseButton::Navigate(NavigationDirection::Forward),
            cx.listener(|this, _, window, cx| {
                this.command(crate::controls::Command::Forward, window, cx);
            }),
        )
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .map(|root| self.with_backdrop(root))
        .text_color(rgb(self.theme.foreground))
        .text_font(&self.config.ui)
        .text_size(px(self.config.ui.size))
        // Text sets from the side it starts, the right in Hebrew; the
        // terminal paints its own cells and never follows.
        .text_start()
    }

    /// The bars the window stacks over the body, and the overlays over them.
    fn root_body(
        &mut self,
        root: Div,
        body: Body,
        zen: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let Body {
            content,
            fill,
            merged,
            sidebar,
        } = body;
        root.when(crate::menus::IN_WINDOW && !zen, |root| {
            root.child(self.render_application_bar(window, cx))
        })
        .when(!merged && !zen, |root| {
            root.child(self.render_titlebar(window, cx))
        })
        .when(zen, |root| {
            root.children(crate::titlebar::zen_inset(fill.ground(), window))
        })
        .child(
            div()
                .debug_selector(|| "window-body".into())
                // The sidebar leads, on the right in a right-to-left
                // language; the terminal area beside it never mirrors.
                .row()
                .flex_1()
                .min_h_0()
                .relative()
                .children(sidebar.map(|(sidebar, width)| {
                    let column = match merged {
                        true => self.sidebar_column(sidebar, window, cx).into_any_element(),
                        false => sidebar.into_any_element(),
                    };
                    self.sliding_sidebar(column, width)
                }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .relative()
                        .child(content)
                        .children(self.render_notices(cx))
                        .children(self.zen_status_bar(cx)),
                ),
        )
        // After the body, whose rows reserve the marks it draws, and under
        // everything that covers the body.
        .child(AnyView::from(self.mark_layer.clone()))
        // At the window's foot in every layout: under the traffic lights
        // it would hide them, and one place is easier to find.
        .when(!zen, |root| root.children(self.render_worktree_banner()))
        .children(self.render_tab_ghost(cx))
        .children(self.render_toasts(window, cx))
        .children(self.render_file_transfer(window, cx))
        .children(self.render_menu_leaving(window, cx))
        .when(self.menu.page.is_some(), |root| {
            root.child(self.render_menu(window, cx))
        })
    }
}

/// What the body made: the content, whether the strip stands in for the
/// title bar, and the sidebar to slide beside it.
struct Body {
    content: AnyElement,
    fill: crate::backdrop::Fill,
    merged: bool,
    sidebar: Option<(ViewElement<AnyView>, f32)>,
}

/// The groups this frame lays out: what each holds, and the sidebar beside
/// them.
struct Areas {
    cell_height: f32,
    fill: crate::backdrop::Fill,
    parked_font: Font,
    shown: Vec<Shown>,
    sidebar: Option<(ViewElement<AnyView>, f32)>,
    slots: Vec<Slot>,
    terminal_slot: Option<Slot>,
    window_frame: Option<crate::presentation::Picture>,
}

/// What the cells paint this frame, measured before anything is built.
struct Paint {
    cell_height: f32,
    cell_width: f32,
    chats: Vec<AnyElement>,
    copy_badge: Option<Div>,
    entity: Entity<SpaceportWindow>,
    fill: crate::backdrop::Fill,
    find_bar: Option<Div>,
    focus: FocusHandle,
    font: Font,
    ground: Ground,
    highlights: Vec<terminal_painter::Highlight>,
    images: Option<Arc<spaceport_client::SurfaceImages>>,
    link_rows: Vec<(u16, std::ops::Range<u16>)>,
    link_shift: Option<(Bounds<Pixels>, Pixels)>,
    marked: Option<(String, Option<Arc<PaneSurfaceFrame>>, Pixels)>,
    menu_open: bool,
    metrics: CellMetrics,
    padding: crate::config::Padding,
    painter: Rc<RefCell<terminal_painter::TerminalPainter>>,
    popup_highlights: Vec<terminal_painter::Highlight>,
    regions: Vec<AnyElement>,
    surface: Option<Arc<PaneSurfaceFrame>>,
    whole: bool,
}
