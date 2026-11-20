// ABOUTME: Draws tab strips and routes tab interactions to their owners.
// ABOUTME: Shares window chrome without changing runtime-owned terminal state.
//! Each editor group's tab strip and frame, and the row of groups. Every
//! group lists the same tabs, as an editor's groups do; which one a group
//! shows is its own choice.
// i18n: migrated

mod agent_title;
mod curved;
mod dividers;
mod launcher;
mod strip;

use super::{SheprdWindow, tab_drag::StripDrag};
use crate::i18n::fl;
#[cfg(test)]
use crate::tab_layout::{DOT_GAP, Lead, ZOOM_ICON};
use crate::{
    TAB_HEIGHT,
    browser::{Fold, GroupId, Leaving, Pick, Shown, Slot},
    herdr_settings::TabBarPosition,
    icons::AgentIcon,
    sidebar::Indicators,
    tab_layout::{Mark, TabContext, TabLayout, TabShape, TabState},
    titlebar::Ends,
};
use gpui::{prelude::*, *};
#[cfg(test)]
use sheprd_client::protocol::AgentStatus;

pub(super) use dividers::Reach;

/// How far a lifted tab's card reaches past each side of the tab.
const LIFT_GROW: f32 = 5.;

impl SheprdWindow {
    /// The layout View > Tab Layout picked.
    pub(crate) fn tab_layout(&self) -> &'static dyn TabLayout {
        crate::tab_layout::layout_for(self.config.tab_layout)
    }

    /// What every tab of `slot`'s strip shares.
    pub(crate) fn tab_context(&self, slot: Slot, indicators: Indicators) -> TabContext<'_> {
        TabContext {
            theme: &self.theme,
            font: &self.config.tabs,
            indicators,
            fill: self.fill(),
            slot,
            ring: (self.chrome_region == Some(crate::window::chrome_focus::Region::TitleBar))
                .then(|| self.theme.primary()),
        }
    }

    /// How a tab of `group` is marked: whether it is the one shown, and
    /// whether the group has the keyboard, so a split shows which one does.
    pub(crate) fn tab_state(&self, selected: bool, group: GroupId) -> TabState {
        TabState {
            selected,
            active: !self.is_split() || self.active_group() == Some(group),
        }
    }

    /// A tab's colors, as the layout in use draws them.
    pub(crate) fn tab_colors(&self, selected: bool, group: GroupId) -> (u32, u32) {
        self.tab_layout()
            .colors(self.tab_state(selected, group), &self.theme)
    }

    /// How wide a tab with `label` draws, as its padding, gaps, close
    /// button, and whatever leads its title add up around the shaped text.
    fn tab_width(
        &self,
        label: &SharedString,
        shape: TabShape,
        indicators: Indicators,
        window: &Window,
    ) -> f32 {
        let run = TextRun {
            len: label.len(),
            font: crate::config::font_for(&self.config.tabs),
            color: black(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let text = f32::from(
            window
                .text_system()
                .shape_line(label.clone(), px(self.config.tabs.size), &[run], None)
                .width,
        );
        // GPUI rounds the text element's intrinsic width up during layout.
        self.tab_layout()
            .width(text.ceil(), shape, &self.config.tabs, indicators)
    }

    /// A closed tab where it stood, narrowing and fading out.
    fn leaving_tab(&self, slot: Slot, leaving: &Leaving, now: std::time::Instant) -> AnyElement {
        let left = leaving.left(now);
        div()
            .debug_selector(move || slot.selector("leaving-tab"))
            .flex_none()
            .h_full()
            .w(px(leaving.width * left))
            .overflow_hidden()
            .opacity(left)
            .flex()
            .items_center()
            .pl(px(12.))
            .whitespace_nowrap()
            .border_r_1()
            .border_color(rgb(self.theme.active))
            .bg(self.fill().over(self.theme.surface, self.theme.surface))
            .text_color(rgb(self.theme.muted))
            .child(leaving.label.clone())
            .into_any_element()
    }

    /// Height of a group's tab strip, which grows with the tab font.
    pub(crate) fn tab_strip_height(&self) -> f32 {
        crate::titlebar::strip_height(
            (self.config.tabs.size * 1.6 + 4.).max(TAB_HEIGHT),
            self.tab_bar_position() == TabBarPosition::Top,
        )
    }

    /// `ends` says which of the header's parts the strip carries, when it
    /// stands in for the header.
    ///
    /// Closes a strip's tab as its close button does: a page at once and a
    /// Herdr tab through its confirmation.
    pub(crate) fn close_strip_tab(
        &mut self,
        pick: Pick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match pick {
            Pick::Herdr(tab) => self.open_tab_close(&tab, window, cx),
            Pick::Page(id) => self.close_browser_tab(id, window, cx),
        }
    }

    /// A tab as the strip places it: armed to lift for reordering, slid
    /// aside for a drop in preview, or carried over the strip, grown so it
    /// reads as held.
    fn draggable_tab(
        &self,
        slot: Slot,
        pick: Pick,
        tab: Stateful<Div>,
        selected: bool,
        drag: Option<&StripDrag>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let shift = drag
            .and_then(|drag| drag.shifts.get(&pick))
            .copied()
            .unwrap_or(0.);
        let grown = drag
            .and_then(|drag| drag.carried.as_ref())
            .and_then(|(carried, grown)| (*carried == pick).then_some(*grown));
        let press = pick.clone();
        let close = pick.clone();
        let tab = tab
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    if event.click_count == 1 {
                        this.press_tab(slot.id, &press, event.position, cx);
                    }
                }),
            )
            // Middle-click closes, as a browser's tabs do.
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.close_strip_tab(close.clone(), window, cx);
                }),
            )
            .when(shift != 0., |tab| tab.relative().left(px(shift)));
        let Some(grown) = grown else {
            return tab.into_any_element();
        };
        let (background, _) = self.tab_colors(selected, slot.id);
        let (wide, tall) = (LIFT_GROW * grown, LIFT_GROW * 0.5 * grown);
        let selector = match &pick {
            Pick::Herdr(id) => format!("lifted-tab-{id}"),
            Pick::Page(id) => format!("lifted-browser-tab-{id}"),
        };
        // A card behind the tab, a little larger than it in the tab's own
        // color, grows as it lifts, so the held tab reads as picked up while
        // its label stays where it was. The tab keeps its place in the row,
        // so the others hold still, and paints last, over the tabs it passes.
        let card = div()
            .debug_selector(move || slot.selector(&selector))
            .absolute()
            .left(px(-wide))
            .right(px(-wide))
            .top(px(-tall))
            .bottom(px(-tall))
            .rounded(px(crate::config::corners::CONTROL))
            .bg(rgb(background))
            .border_1()
            .border_color(rgb(self.theme.active))
            .shadow_lg();
        deferred(
            div()
                .flex_none()
                // A flex box, so the held tab keeps the strip's full height.
                .flex()
                .relative()
                .left(px(shift))
                .cursor_grabbing()
                .child(card)
                .child(tab.left(px(0.)).border_color(rgb(background))),
        )
        .with_priority(1)
        .into_any_element()
    }

    /// Follows the pointer anywhere in the window while a tab drag is armed,
    /// so the drag carries on past the strip, and a lifted tab's release is
    /// its drop rather than a click on whatever is under it.
    fn tab_drag_listeners(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                let moving = view.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture {
                        let viewport = window.viewport_size();
                        moving.update(cx, |this, cx| {
                            let held = event.pressed_button == Some(MouseButton::Left);
                            if this.move_tab_drag(event.position, held, viewport, cx) {
                                cx.stop_propagation();
                            }
                            #[cfg(target_os = "macos")]
                            this.float_tab(window, cx);
                        });
                    }
                });
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                        view.update(cx, |this, cx| {
                            if this.release_tab_drag(window, cx) {
                                cx.stop_propagation();
                            }
                        });
                    }
                });
            },
        )
        .absolute()
        .size_full()
    }

    fn strip_button(
        &self,
        slot: Slot,
        (id, label): (&'static str, String),
        icon: &'static str,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        div()
            .id(SharedString::from(slot.selector(id)))
            .debug_selector(move || slot.selector(id))
            .role(Role::Button)
            .aria_label(label)
            .map(|control| self.chrome_stop(crate::window::chrome_focus::Region::TitleBar, control))
            .w(px(30.))
            .min_h(px(TAB_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(self.theme.active)))
            .child(
                svg()
                    .path(icon)
                    .size(px(14.))
                    .text_color(rgb(self.theme.muted)),
            )
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, event, window, cx| {
                cx.stop_propagation();
                action(this, event, window, cx);
            }))
    }

    /// Stands in for a tab the group's connection has not reached yet, or
    /// for nothing: a group without tabs says how to open one. Pressing the
    /// group, as the button invites, gives it the window's connection.
    pub(super) fn render_stand_in(
        &self,
        slot: Slot,
        shown: &Shown,
        keyboard: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (title, note): (SharedString, String) = match shown {
            Shown::Elsewhere(tab) => {
                let label = self
                    .live
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| {
                        snapshot
                            .tabs
                            .iter()
                            .find(|candidate| &candidate.tab_id == tab)
                            .map(|tab| self.strip_label(snapshot, tab))
                    })
                    .unwrap_or_else(|| tab.clone().into());
                (label, fl!("tab-strip-opening"))
            }
            // A terminal group whose connection has not shown its tab yet.
            Shown::Terminal => ("".into(), fl!("tab-strip-opening")),
            // What a group without tabs says, where its tab would show.
            Shown::Empty if self.is_split() && self.group_tabs(slot.id, cx).is_empty() => (
                fl!("tab-strip-no-tabs").into(),
                fl!("tab-strip-no-tabs-hint"),
            ),
            Shown::Page(_) | Shown::Empty => ("".into(), fl!("tab-strip-nothing")),
        };
        let elsewhere = matches!(shown, Shown::Elsewhere(_));
        crate::config::pad(div(), self.config.terminal_padding)
            .id(SharedString::from(slot.selector("stand-in")))
            .debug_selector(move || slot.selector("stand-in"))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .items_center()
            .justify_center()
            .gap(px(10.))
            .bg(self.fill().base(self.theme.background))
            .text_color(rgb(self.theme.muted))
            .when(keyboard, |stand_in| stand_in.track_focus(&self.focus))
            .when(!title.is_empty(), |stand_in| {
                stand_in.child(
                    div()
                        .max_w_full()
                        .truncate()
                        .text_color(rgb(self.theme.foreground))
                        .child(title),
                )
            })
            .child(div().max_w(px(360.)).px_4().text_center().child(note))
            // The group in use carries the window's flash, as a page's
            // toolbar and the terminal do.
            .children(
                self.flash_shown()
                    .filter(|_| self.active_group() == Some(slot.id))
                    .map(|(flash, shown)| {
                        div()
                            .debug_selector(|| "flash".into())
                            .opacity(shown)
                            .max_w_full()
                            .truncate()
                            .text_color(rgb(flash.accent(&self.theme)))
                            .child(flash.text.clone())
                    }),
            )
            .when(elsewhere, |stand_in| {
                stand_in.child(
                    div()
                        .id(SharedString::from(slot.selector("show-here")))
                        .debug_selector(move || slot.selector("show-here"))
                        .px(px(10.))
                        .py(px(5.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .border_1()
                        .border_color(rgb(self.theme.active))
                        .text_color(rgb(self.theme.foreground))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(self.theme.active)))
                        .child(fl!("tab-strip-show-here"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_group(slot.id, window, cx);
                        })),
                )
            })
            .into_any_element()
    }

    /// Which side of a group's content its strip sits on.
    fn strip_side(&self) -> curved::Side {
        match self.tab_bar_position() {
            TabBarPosition::Bottom => curved::Side::Below,
            TabBarPosition::Top => curved::Side::Above,
        }
    }

    /// Where Herdr's shared config puts the tab row.
    pub(crate) fn tab_bar_position(&self) -> TabBarPosition {
        self.settings
            .shared
            .as_ref()
            .map(|shared| shared.tab_bar_position)
            .unwrap_or_default()
    }

    /// Whether `group`'s strip steps aside: always in zen mode, else under Herdr's
    /// `hide_tab_bar_when_single_tab`: only when the window is not split, so
    /// every group keeps the strip that names it and takes dropped tabs, and
    /// only while the strip would list one tab at most, browser tabs counted.
    pub(crate) fn strip_hidden(&self, group: GroupId, cx: &App) -> bool {
        self.zen_layout()
            || self
                .settings
                .shared
                .as_ref()
                .is_some_and(|shared| shared.hide_tab_bar_when_single_tab)
                && !self.is_split()
                && self.folding_groups().is_empty()
                && !self.tab_drag_in(group)
                && self.group_tabs(group, cx).len() <= 1
    }

    /// One group: its strip above or below what it shows, as Herdr's
    /// `tab_bar_position` places it. Pressing anywhere in it makes it the
    /// group in use. `ends` is set while the strips stand in for the header.
    /// `reach` is how far its dividers' grips lie over it.
    pub(super) fn render_group(
        &mut self,
        slot: Slot,
        body: AnyElement,
        ends: Option<Ends>,
        reach: Reach,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strip = (!self.strip_hidden(slot.id, cx))
            .then(|| self.render_tab_strip(slot, ends, window, cx));
        let strip_side = strip.is_some().then(|| self.strip_side());
        let bottom = self.tab_bar_position() == TabBarPosition::Bottom;
        let linked = self
            .live
            .snapshot
            .as_deref()
            .is_some_and(crate::worktree_identity::focused_is_linked);
        let rule = crate::worktree_identity::rule(&self.config.worktrees, linked, &self.theme);
        let rule = rule.map(|rule| rule.debug_selector(move || slot.selector("worktree-rule")));
        let strip = crate::worktree_identity::place_rule(strip, rule, bottom);
        // Zen mode draws the one group in use across the whole row, at once
        // even while it is still opening.
        let zen = self.zen_layout();
        let share = (!zen && (self.is_split() || !self.folding_groups().is_empty()))
            .then(|| self.group_share(slot.id));
        let opened = self.group_opened(slot.id).filter(|_| !zen);
        // The body's ground, beside the strip rather than under it, so the
        // strip's own color is no more solid than the rest of the window.
        let body = self.with_ground(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .child(body),
            self.fill().ground(),
        );
        let body = self.content_card(slot, body, strip_side);
        let column = div()
            .id(SharedString::from(slot.selector("group")))
            .debug_selector(move || slot.selector("group"))
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0()
            .h_full()
            // Opening, the group is laid out at the width it opens to and
            // uncovered from the right, so it slides in rather than squeezing,
            // and its terminal keeps one size while it does.
            .map(|column| match opened {
                Some(k) => column.flex_none().w(relative(1. / k.max(0.02))),
                None => column.w_full(),
            })
            .capture_any_mouse_down(cx.listener(move |this, _, window, cx| {
                if this.menu.page.is_none() {
                    this.activate_group(slot.id, window, cx);
                }
            }))
            .map(|column| match (strip, bottom) {
                (Some(strip), false) => column.child(strip).child(body),
                (Some(strip), true) => column.child(body).child(strip),
                (None, _) => column.child(body),
            })
            .child({
                let drawn = self.browser.group_bounds.clone();
                canvas(
                    move |bounds, _, _| {
                        drawn.borrow_mut().insert(slot.id, bounds);
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full()
            })
            // A tab carried over the group lands in it when dropped.
            .when(self.drop_group() == Some(slot.id), |column| {
                let accent = crate::menu::accent(&self.theme);
                column.child(
                    div()
                        .debug_selector(move || slot.selector("tab-drop"))
                        .absolute()
                        .inset_0()
                        .border_2()
                        .border_color(accent)
                        .bg(accent.opacity(0.12)),
                )
            });
        div()
            .relative()
            .flex()
            .justify_end()
            .overflow_hidden()
            .min_w_0()
            .min_h_0()
            .map(|frame| match share {
                // Dividers take their width from every group alike.
                Some(share) => frame.flex_shrink(1.).w(relative(share)),
                None => frame.flex_1(),
            })
            .child(column)
            // Last, so the grips lie over what the group shows.
            .children(reach.grips(slot.index, window, cx))
            .into_any_element()
    }

    /// A closed group folding away to the right where it stood: its strip's
    /// band and an empty body.
    fn folding_group(&self, fold: &Fold) -> AnyElement {
        div()
            .debug_selector(|| "folding-group".into())
            .flex_shrink(1.)
            .w(relative(fold.share))
            .h_full()
            .overflow_hidden()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(self.theme.active))
            .when(self.tab_bar_position() == TabBarPosition::Bottom, |group| {
                group.flex_col_reverse()
            })
            .child(
                div()
                    .flex_none()
                    .h(px(self.tab_strip_height()))
                    .bg(self.fill().panel(self.theme.surface)),
            )
            .child(div().flex_1().bg(self.fill().ground()))
            .into_any_element()
    }
}

/// What leads a Herdr tab: its status, and the agent running in it, the
/// focused one when several do.
fn herdr_mark(
    snapshot: &sheprd_client::protocol::ClientShellSnapshot,
    tab: &sheprd_client::protocol::ClientShellTab,
) -> Mark {
    let agent = snapshot
        .agents
        .iter()
        .filter(|agent| agent.tab_id == tab.tab_id)
        .max_by_key(|agent| agent.focused)
        .map(|agent| AgentIcon::from_identity(agent.agent.as_deref()));
    Mark::Herdr {
        status: tab.agent_status,
        agent,
    }
}

#[cfg(test)]
mod tests;
