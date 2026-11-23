//! The agents list: how it is sorted, how each agent's marks are drawn, and
//! the words its status is shown in. Status comes from the daemon's snapshot,
//! never from guessing at terminal output. The model the marks and words are
//! made of — the indicator colors and sizes, the daemon's status names, and
//! how a place, pane and label are read from a snapshot — lives in
//! `spaceport_sidebar::agents`; what is left here paints it and words it.
//! i18n: migrated

pub(super) mod sections;

use super::{label_text, line_height, unknown_dot};
use crate::direction::Logical;
use crate::i18n::fl;
use crate::preferences::AgentSort;
use crate::{
    SpaceportWindow,
    config::{FontConfig, StatusIcons},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::{AgentStatus, ClientShellAgent};

pub(crate) use spaceport_sidebar::agents::{
    Indicators, agent_labels, agent_place, icon_size, pane_name, sorted_agents, state_label,
    status_style, tile_size,
};

pub(super) fn agents_sort(
    window: &SpaceportWindow,
    cx: &mut Context<SpaceportWindow>,
) -> Stateful<Div> {
    let theme = &window.theme;
    let view = window
        .live
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.agent_view_label.clone());
    let label = view
        .clone()
        .unwrap_or_else(|| sort_label(window.agent_sort));
    div()
        .id("agents-sort")
        .debug_selector(|| "agents-sort".into())
        // A plugin's view only names itself; the sort is a button.
        .map(|sort| match &view {
            Some(view) => sort
                .role(Role::Label)
                .aria_value(fl!("sidebar-agents-view", view = view.as_str())),
            None => window.chrome_stop(
                crate::window::chrome_focus::Region::Sidebar,
                sort.role(Role::Button).aria_label(fl!(
                    "sidebar-agents-sort",
                    sort = sort_label(window.agent_sort)
                )),
            ),
        })
        .flex_none()
        .min_w_0()
        .truncate()
        .text_color(rgb(theme.muted))
        .when(view.is_none(), |sort| {
            sort.cursor_pointer()
                .hover(|style| style.text_color(rgb(theme.foreground)))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.agent_sort = this.agent_sort.toggled();
                    this.agent_sort_modified = true;
                    this.save_chrome();
                    cx.notify();
                }))
        })
        .child(label_text(&label))
}

/// The agents header's name for how it sorts, which a click toggles.
fn sort_label(sort: AgentSort) -> String {
    match sort {
        AgentSort::Grouped => fl!("sidebar-sort-grouped"),
        AgentSort::Priority => fl!("sidebar-sort-priority"),
    }
}

/// Every listed host's agents in the order the panel paints them, with the
/// host's endpoint index. Grouped lists one host's agents after another's, in
/// sidebar order. Priority merges hosts as Herdr's terminal client does: a
/// disconnected host's last-known agents after connected ones, then attention,
/// then the most recent change on the clock every connection shares, so a
/// blocked agent on one host rises above an idle one on another. A plugin
/// view orders only its own host's agents, so while any host shows one, each
/// host keeps its own order.
pub(super) fn panel_agents<'a>(
    hosts: impl IntoIterator<Item = (usize, &'a crate::state::LiveState)>,
    sort: AgentSort,
    now: u64,
) -> Vec<(usize, &'a ClientShellAgent)> {
    let mut merge = sort == AgentSort::Priority;
    let mut rows = Vec::new();
    for (index, live) in hosts {
        let Some(snapshot) = live.snapshot.as_deref() else {
            continue;
        };
        merge &= snapshot.agent_view_label.is_none();
        rows.extend(
            sorted_agents(snapshot, sort)
                .into_iter()
                .map(|agent| (index, live, agent)),
        );
    }
    if merge {
        rows.sort_by_cached_key(|(index, live, agent)| sections::order(*index, live, agent, now));
    }
    rows.into_iter()
        .map(|(index, _, agent)| (index, agent))
        .collect()
}

pub(crate) use spaceport_client::text::agent_name;

/// The edge rail's bar on a row's leading edge, for the statuses worth
/// noticing. Absolutely placed, so the row must be `relative`.
pub(super) fn rail(indicators: Indicators, status: AgentStatus, key: &str) -> Option<Div> {
    if !indicators.edge_rail
        || !matches!(
            status,
            AgentStatus::Working | AgentStatus::Done | AgentStatus::Blocked
        )
    {
        return None;
    }
    Some(
        div()
            .debug_selector(|| format!("rail-{key}"))
            .absolute()
            .inset_s(px(0.))
            .top(relative(RAIL_INSET))
            .bottom(relative(RAIL_INSET))
            .w(px(RAIL_WIDTH))
            .rounded_e(px(RAIL_WIDTH))
            .bg(rgb(indicators.color(status))),
    )
}

const RAIL_WIDTH: f32 = 3.;
/// The bar's inset from the row's top and bottom, as a share of its height,
/// so even a one-line Compact row at the smallest font keeps a visible bar.
const RAIL_INSET: f32 = 0.2;

pub(crate) fn status_indicator(
    status: AgentStatus,
    font: &FontConfig,
    indicators: Indicators,
) -> Div {
    status_mark(status, font, indicators, indicators.color(status), false)
}

/// Configured tokens can override color and weight without changing the set;
/// only dots have a weight to change.
pub(super) fn status_mark(
    status: AgentStatus,
    font: &FontConfig,
    indicators: Indicators,
    color: u32,
    bold: bool,
) -> Div {
    if indicators.icons != StatusIcons::Dots {
        return status_icon(status, font, indicators, color);
    }
    // Upstream dots: blocked/done filled, idle hollow, unknown a small dot.
    // Working spins instead, so busy agents read apart from settled ones.
    // They grow with the font, as upstream's dot glyphs do.
    let dot = indicators.width(font);
    let (diameter, filled) = match status {
        AgentStatus::Unknown => (unknown_dot(dot), true),
        AgentStatus::Idle => (dot, false),
        _ => (dot, true),
    };
    div()
        .w(px(dot))
        .h(px(dot))
        .mt(px((line_height(font) - dot) / 2.))
        .flex_none()
        .row()
        .items_center()
        .justify_center()
        .when(status == AgentStatus::Working, |slot| {
            slot.child(icon_mark(
                StatusIcons::Dots,
                status,
                dot,
                color,
                indicators.fade(),
            ))
        })
        .when(status != AgentStatus::Working, |slot| {
            slot.child(
                div()
                    .size(px(if bold { dot } else { diameter }))
                    .rounded_full()
                    .border_1()
                    .when(bold, |dot| dot.border_2())
                    .border_color(rgb(color))
                    .when(filled, |dot| dot.bg(rgb(color))),
            )
        })
}

/// A working dot: a faint ring with a spinning arc, `size` across. Rows draw
/// it at the dot's size, so they keep their geometry.
fn working_orbit(size: f32, color: u32) -> impl IntoElement {
    spin(
        svg()
            .debug_selector(|| "status-orbit".into())
            .path("icons/status-orbit.svg")
            .size(px(size))
            .flex_none()
            .text_color(rgb(color)),
        "status-orbit",
        900,
    )
}

/// Turns `mark` once every `period_ms`. Every spinner of one kind shares an
/// id and the app's synced clock, so all working agents turn in step, and
/// repaints are capped because a sidebar can show many at once. GPUI holds
/// it still under reduced motion.
fn spin(mark: Svg, id: &'static str, period_ms: u64) -> impl IntoElement {
    mark.with_animation(
        id,
        Animation::new(crate::motion::duration(std::time::Duration::from_millis(
            period_ms,
        )))
        .repeat_synced()
        .with_max_fps(crate::mark_layer::MAX_FPS),
        |mark, delta| mark.with_transformation(Transformation::rotate(percentage(delta))),
    )
}

/// A drawn status icon, centered in its slot. Configured weights do not
/// apply: the icon's strokes are fixed by its asset.
fn status_icon(status: AgentStatus, font: &FontConfig, indicators: Indicators, color: u32) -> Div {
    let (icons, fade) = (indicators.icons, indicators.fade());
    let size = icon_size(font);
    let slot = div().flex_none().row().items_center().justify_center();
    if icons != StatusIcons::Tiles {
        return slot
            .size(px(size))
            .child(icon_mark(icons, status, size, color, fade));
    }
    slot.size(px(tile_size(font)))
        .rounded(px(crate::config::corners::SMALL))
        .bg(rgba((color << 8) | 0x30))
        .border_1()
        .border_color(rgba((color << 8) | 0x70))
        .child(icon_mark(icons, status, size - 4., color, fade))
}

/// One set's mark for `status` at `size`: its asset, turning while working,
/// or the mark drawn for the sets without assets. The working assets are
/// rings around a turnable center, so spinning the whole icon keeps its
/// outline still. A mark that moves draws in the window's mark layer, so its
/// frames never rebuild the sidebar; `fade` is the opacity of its row.
pub(super) fn icon_mark(
    icons: StatusIcons,
    status: AgentStatus,
    size: f32,
    color: u32,
    fade: f32,
) -> AnyElement {
    // Working always moves; of the settled statuses only a drawn blocked
    // mark can.
    let moves = status == AgentStatus::Working
        || (status == AgentStatus::Blocked && crate::icons::status_icon(icons, status).is_none());
    if !moves {
        return drawn_mark(icons, status, size, color);
    }
    crate::mark_layer::layered(size, fade, move || drawn_mark(icons, status, size, color))
        .into_any_element()
}

fn drawn_mark(icons: StatusIcons, status: AgentStatus, size: f32, color: u32) -> AnyElement {
    if icons == StatusIcons::Dots && status == AgentStatus::Working {
        return working_orbit(size, color).into_any_element();
    }
    let Some(path) = crate::icons::status_icon(icons, status) else {
        return super::animated::mark(icons, status, size, color);
    };
    let mark = svg()
        .path(path)
        .size(px(size))
        .flex_none()
        .text_color(rgb(color));
    if status != AgentStatus::Working {
        return mark.into_any_element();
    }
    spin(
        mark.debug_selector(|| "status-spin".into()),
        "status-spin",
        1400,
    )
    .into_any_element()
}

/// The word the daemon's `state_text` token shows for a status when its
/// sidebar config asks for it, in the app's language. Rules match
/// the daemon's status key instead, so they hold in every language.
pub(crate) fn status_text(status: AgentStatus) -> String {
    match status {
        AgentStatus::Working => fl!("sidebar-status-working"),
        AgentStatus::Blocked => fl!("sidebar-status-blocked"),
        AgentStatus::Done => fl!("sidebar-status-done"),
        AgentStatus::Idle => fl!("sidebar-status-idle"),
        AgentStatus::Unknown => fl!("sidebar-status-unknown"),
    }
}

#[cfg(test)]
mod tests;
