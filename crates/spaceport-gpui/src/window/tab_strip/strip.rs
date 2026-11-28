//! Drawing one group's strip: what it lists and where each tab sits, the
//! tabs it drew with their closed neighbours, and the bar around them. Each
//! phase reads what the one before it measured rather than measuring again.

use super::herdr_mark;
use crate::browser::{Listed, Pick, Slot, ThumbDrag};
use crate::controls::Command;
use crate::fonts::StyledFont;
use crate::i18n::fl;
use crate::notification_mutes::Silenced;
use crate::sidebar::Indicators;
use crate::tab_layout::{Lead, Probe, TabContext, TabFace, TabShape};
use crate::titlebar::Ends;
use crate::window::SpaceportWindow;
use gpui::{prelude::*, *};
use spaceport_client::protocol::ClientShellSnapshot;
use std::collections::HashSet;
use std::time::Instant;

/// How tall the thumb along a scrolling strip's foot draws.
const THUMB_HEIGHT: f32 = 4.;

/// How opaque the thumb draws over the strip's muted color, and while held.
const THUMB_ALPHA: u32 = 0x80;
const THUMB_HELD_ALPHA: u32 = 0xc0;

impl SpaceportWindow {
    /// The strip in three phases: what it lists, the tabs it drew, and the
    /// bar around them.
    pub(super) fn render_tab_strip(
        &mut self,
        slot: Slot,
        ends: Option<Ends>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let measured = self.strip_measure(slot, window, cx);
        let scroller = self.strip_scroller(slot, &measured, window, cx);
        self.strip_bar(slot, ends, scroller, window, cx)
    }

    /// What the strip lists before it draws: the tabs in their places, the
    /// chosen one's breadth, and the row it remembers for the next frame.
    fn strip_measure(&mut self, slot: Slot, window: &Window, cx: &App) -> StripMeasure {
        let pick = self.group_pick(slot.id);
        let indicators = Indicators::new(
            self.settings.shared.as_ref(),
            self.config.status_icons,
            self.config.edge_rail,
            matches!(
                cx.window_appearance(),
                WindowAppearance::Light | WindowAppearance::VibrantLight
            ),
            &self.theme,
        );
        // The tabs the user muted, or whose workspace they muted, read from
        // a handle of their own so the strip can change below.
        let shown = self.live.snapshot.clone();
        let silenced = shown
            .as_deref()
            .map(|snapshot| {
                let daemon =
                    crate::notification_mutes::Daemon::of(&self.endpoints[self.selected_endpoint]);
                crate::notification_mutes::Mutes::silenced_in(cx, &daemon, snapshot)
            })
            .unwrap_or_default();
        // Resolved once: both phases ask the same question of every tab.
        let muted = StripMutes::resolve(&silenced, self.live.snapshot.as_deref());
        // What the strip lists, measured, so the ones that just opened grow
        // in and the ones that just closed shrink out where they stood.
        let now = crate::motion::now();
        let listed = self.strip_listed(slot, pick.as_ref(), indicators, &muted, window, cx);
        // Leaving tabs slot in among these below; `None` marks where.
        let places = listed.iter().map(|tab| Some(tab.pick.clone())).collect();
        // The chosen tab's width, which changes as a page's title comes in.
        let chosen_width = pick
            .as_ref()
            .and_then(|pick| listed.iter().find(|tab| &tab.pick == pick))
            .map_or(0., |tab| tab.width);
        self.observe_strip(slot.id, listed, now);
        StripMeasure {
            chosen_width,
            indicators,
            muted,
            now,
            pick,
            places,
        }
    }

    /// Every tab the strip lists, with its label and the lead its mark draws.
    fn strip_listed(
        &self,
        slot: Slot,
        pick: Option<&Pick>,
        indicators: Indicators,
        muted_tabs: &StripMutes,
        window: &Window,
        cx: &App,
    ) -> Vec<Listed> {
        let herdr: Vec<((Pick, SharedString), Lead, bool, bool)> = self
            .live
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .tabs
                    .iter()
                    .filter(|t| Some(&t.workspace_id) == snapshot.focused_workspace_id.as_ref())
                    .map(|t| {
                        (
                            (Pick::Herdr(t.tab_id.clone()), self.strip_label(snapshot, t)),
                            herdr_mark(snapshot, t).lead(),
                            t.zoomed,
                            muted_tabs.tab(&t.tab_id),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        herdr
            .into_iter()
            .chain(
                self.browser_tab_labels(cx)
                    .into_iter()
                    .map(|entry| (entry, Lead::Icon, false, false)),
            )
            .filter(|((tab, _), _, _, _)| self.group_lists(slot.id, tab))
            .map(|((tab, label), lead, zoomed, muted)| {
                let shape = TabShape {
                    lead,
                    zoomed,
                    muted,
                    selected: pick == Some(&tab),
                };
                Listed {
                    width: self.tab_width(&label, shape, indicators, window),
                    pick: tab,
                    label,
                }
            })
            .collect()
    }

    /// The tabs the strip drew, each a drop target, with the closed tabs
    /// between them and the scroll thumb over them.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 106 lines")]
    fn strip_scroller(
        &mut self,
        slot: Slot,
        measured: &StripMeasure,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let layout = self.tab_layout();
        let (body, body_placer) = self
            .tab_body(slot, measured.pick.as_ref(), self.strip_side(), window)
            .unzip();
        let faces = self.tab_context(slot, measured.indicators);
        let mut built = self.strip_tabs(slot, measured, &faces, cx);
        // A page that another group shows paints into the chosen tab here.
        let shown = match measured.pick.as_ref() {
            Some(Pick::Page(id)) => Some(*id),
            _ => None,
        };
        built.extend(
            self.browser_tab_entries(slot, shown, &faces, cx)
                .into_iter()
                .map(|(id, tab)| (Pick::Page(id), tab)),
        );
        let mut places = measured.places.clone();
        // Closed tabs go back where they stood, in the order they stood.
        let mut leaving = self.leaving_tabs(slot.id);
        leaving.sort_by_key(|leaving| leaving.index);
        for leaving in &leaving {
            places.insert(leaving.index.min(places.len()), None);
        }
        let scroll = self.strip_scroll(slot.id);
        let drag = self.strip_drag(slot.id, &places, &scroll, measured.now);
        if drag.as_ref().is_some_and(|drag| drag.moving) {
            window.request_animation_frame();
        }
        let mut entries: Vec<AnyElement> = built
            .into_iter()
            .map(|(tab_pick, tab)| {
                let selected = measured.pick.as_ref() == Some(&tab_pick);
                let tab = match &body_placer {
                    Some(placer) if selected => tab.child(placer.target()),
                    _ => tab,
                };
                self.draggable_tab(slot, tab_pick, tab, selected, drag.as_ref(), cx)
            })
            .collect();
        for leaving in leaving {
            let index = leaving.index.min(entries.len());
            entries.insert(index, self.leaving_tab(slot, &leaving, measured.now));
        }
        let tabs = div()
            .id(SharedString::from(slot.selector("tabs")))
            .role(Role::TabList)
            .aria_label(fl!("tab-strip-tabs"))
            .flex()
            .flex_none()
            .h(px(self.tab_strip_height()))
            .text_font(&self.config.tabs)
            .text_size(px(self.config.tabs.size))
            .overflow_x_scroll()
            .track_scroll(&scroll)
            .px(px(layout.padding()))
            .text_color(rgb(self.theme.foreground))
            .children(entries);
        if let Some((index, pick)) = measured.pick.as_ref().and_then(|pick| {
            let index = places
                .iter()
                .position(|place| place.as_ref() == Some(pick))?;
            Some((index, pick))
        }) && self.reveal_tab(slot.id, pick, index, measured.chosen_width)
        {
            window.request_animation_frame();
        }
        let hover_group = SharedString::from(slot.selector("tab-scroller"));
        let thumb_color = |alpha| rgba((self.theme.muted << 8) | alpha);
        // The thumb sits over the tabs rather than inside them, so it stays
        // put as they scroll beneath it; the pointer finds it along the
        // strip's foot, as in an editor.
        div()
            .id(SharedString::from(slot.selector("tab-scroller")))
            .debug_selector(move || slot.selector("tab-scroller"))
            .group(hover_group.clone())
            .relative()
            .flex()
            .flex_shrink_1()
            .min_w_0()
            // The list's fill, here rather than on the list so the body,
            // laid over the list, draws above it.
            .bg(self.fill().over(self.theme.surface, self.theme.surface))
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<ThumbDrag>, _, cx| {
                    let ThumbDrag(group) = *event.drag(cx);
                    if group == slot.id && this.drag_strip_thumb(group, event.event.position.x) {
                        cx.notify();
                    }
                }),
            )
            // Beside the tab list rather than in it, so the body neither
            // counts toward what the list scrolls nor shifts its tabs'
            // indices; first, so every tab draws over it.
            .children(body)
            .child(tabs.flex_shrink_1().min_w_0())
            .children(self.strip_thumb(slot.id).map(|thumb| {
                div()
                    .id(SharedString::from(slot.selector("tab-scroll-thumb")))
                    .debug_selector(move || slot.selector("tab-scroll-thumb"))
                    .absolute()
                    .bottom_0()
                    .left(px(thumb.left))
                    .w(px(thumb.width))
                    .h(px(THUMB_HEIGHT))
                    // The wheel still scrolls the strip over the thumb.
                    .block_mouse_except_scroll()
                    .group_hover(hover_group, |s| s.bg(thumb_color(THUMB_ALPHA)))
                    .hover(|s| s.bg(thumb_color(THUMB_ALPHA)))
                    .active(|s| s.bg(thumb_color(THUMB_HELD_ALPHA)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _, _| {
                            this.grab_strip_thumb(slot.id, event.position.x);
                        }),
                    )
                    .on_drag(ThumbDrag(slot.id), |_, _, _, cx| cx.new(|_| EmptyView))
            }))
    }

    /// The Herdr tabs this strip lists, each with the menu and click that
    /// choose it.
    fn strip_tabs(
        &self,
        slot: Slot,
        measured: &StripMeasure,
        faces: &TabContext<'_>,
        cx: &mut Context<Self>,
    ) -> Vec<(Pick, Stateful<Div>)> {
        let layout = self.tab_layout();
        let mut built: Vec<(Pick, Stateful<Div>)> = Vec::new();
        let Some(snapshot) = self.live.snapshot.as_deref() else {
            return built;
        };
        for tab in snapshot.tabs.iter().filter(|t| {
            Some(&t.workspace_id) == snapshot.focused_workspace_id.as_ref()
                && self.group_lists(slot.id, &Pick::Herdr(t.tab_id.clone()))
        }) {
            let id = tab.tab_id.clone();
            let context_id = id.clone();
            let close_id = id.clone();
            let selected = matches!(&measured.pick, Some(Pick::Herdr(picked)) if *picked == id);
            let muted = measured.muted.tab(&id);
            let label = self.strip_label(snapshot, tab);
            let base = div()
                .id(SharedString::from(format!("tab-{id}")))
                .debug_selector({
                    let id = id.clone();
                    move || slot.selector(&format!("tab-{id}"))
                })
                .role(Role::Tab)
                .aria_label(if muted {
                    fl!("tab-strip-muted", label = label.as_ref())
                } else {
                    label.to_string()
                })
                .aria_selected(selected)
                .map(|control| faces.stop(control));
            let face = TabFace {
                label,
                mark: herdr_mark(snapshot, tab),
                zoomed: tab.zoomed,
                muted,
                probe: Probe::Herdr(id.clone()),
                close: Box::new(cx.listener(move |this, _, window, cx| {
                    this.close_strip_tab(Pick::Herdr(close_id.clone()), window, cx);
                })),
            };
            built.push((
                Pick::Herdr(id.clone()),
                layout
                    .tab(base, face, self.tab_state(selected, slot.id), faces)
                    .map(|tab| self.grow_tab(tab, slot.id, &Pick::Herdr(id.clone())))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            this.open_tab_menu(
                                &context_id,
                                Some(slot.id),
                                event.position,
                                window,
                                cx,
                            );
                            this.menu.opening_right_click =
                                this.menu.page == Some(crate::menu::Page::Tab);
                        }),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.choose_herdr_tab(slot.id, &id, window, cx);
                    })),
            ));
        }
        built
    }

    /// The bar around the tabs: the drag listeners, the leading controls,
    /// the scroller, and the buttons that follow it.
    fn strip_bar(
        &self,
        slot: Slot,
        ends: Option<Ends>,
        scroller: Stateful<Div>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let layout = self.tab_layout();
        div()
            .flex()
            .flex_none()
            .relative()
            .bg(self.fill().panel(self.theme.surface))
            .text_color(rgb(self.theme.foreground))
            .when(self.tab_drag_in(slot.id), |strip| {
                strip.child(self.tab_drag_listeners(cx))
            })
            .children(
                ends.filter(|ends| ends.leading)
                    .and_then(|_| self.strip_leading(window, cx)),
            )
            .children(self.strip_launcher(slot, super::launcher::Spot::Leading, cx))
            // Tabs size to their content and shrink when the row is full, so
            // the button sits after the last tab instead of at the far right
            // of the window.
            .child(scroller)
            .child(
                self.strip_new_tab(
                    slot,
                    div()
                        .id(SharedString::from(slot.selector("new-tab")))
                        .debug_selector(move || slot.selector("new-tab"))
                        .role(Role::Button)
                        .aria_label(fl!("tab-strip-new-tab"))
                        .map(|control| {
                            self.chrome_stop(crate::window::chrome_focus::Region::TitleBar, control)
                        })
                        .map(|button| {
                            layout.new_tab(
                                button,
                                svg()
                                    .path("icons/plus.svg")
                                    .debug_selector(move || slot.selector("new-tab-icon"))
                                    .size(px(14.))
                                    // Quiet like the unselected tabs beside it.
                                    .text_color(rgb(self.theme.muted))
                                    .into_any_element(),
                                &self.theme,
                            )
                        })
                        // A new Herdr tab opens in the group that asked for it;
                        // other kinds of tab are behind "…".
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.expect_new_tab_in(slot.id);
                            this.command(Command::Tab, window, cx);
                        })),
                    window,
                    cx,
                ),
            )
            .child(self.strip_room(ends, window))
            .children(self.strip_launcher(slot, super::launcher::Spot::Trailing, cx))
            .child(self.strip_button(
                slot,
                ("split-editor", fl!("tab-strip-split")),
                "icons/split.svg",
                cx,
                move |this, _, window, cx| this.split_group(slot.id, window, cx),
            ))
            .child(self.strip_button(
                slot,
                ("tab-actions", fl!("tab-strip-actions")),
                "icons/more.svg",
                cx,
                move |this, event, window, cx| {
                    this.open_group_menu(slot.id, event.position(), window, cx);
                },
            ))
            .when(ends.is_some_and(|ends| ends.trailing), |strip| {
                strip.child(self.strip_trailing(window))
            })
    }
}

/// What one strip measured: the tabs it lists, where each sits, the breadth
/// of the chosen one, and the row it remembers for the next frame.
struct StripMeasure {
    chosen_width: f32,
    indicators: Indicators,
    muted: StripMutes,
    now: Instant,
    pick: Option<Pick>,
    places: Vec<Option<Pick>>,
}

/// The Herdr tabs this frame's mutes silence, answered once so the widths
/// the strip measures and the tabs it draws agree.
#[derive(Default)]
struct StripMutes {
    /// The whole device is muted, which silences every tab.
    host: bool,
    /// Tabs muted on their own, or through their workspace.
    tabs: HashSet<String>,
}

impl StripMutes {
    fn resolve(silenced: &Silenced<'_>, snapshot: Option<&ClientShellSnapshot>) -> Self {
        if silenced.is_empty() {
            return Self::default();
        }
        Self {
            host: silenced.host(),
            tabs: snapshot
                .into_iter()
                .flat_map(|snapshot| snapshot.tabs.iter())
                .filter(|tab| silenced.tab(&tab.tab_id))
                .map(|tab| tab.tab_id.clone())
                .collect(),
        }
    }

    fn tab(&self, id: &str) -> bool {
        self.host || self.tabs.contains(id)
    }
}
