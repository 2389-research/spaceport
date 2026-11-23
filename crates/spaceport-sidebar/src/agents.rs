//! The agents list's model: the colors and sizes its status marks are drawn
//! with, the palette they default to, the daemon's name for a status, how each
//! agent's place, pane and labels are read from a snapshot, and the marks
//! themselves. Status comes from the daemon's snapshot, never from guessing at
//! terminal output. The app words every status, and supplies the one thing a
//! moving mark needs from a window, so the words and the window's own mark
//! layer stay there.

use crate::direction::Logical;
use crate::metrics::{STATUS_DOT_UNKNOWN, STATUS_WIDTH, line_height, scaled, unknown_dot};
use crate::order::status_priority;
use crate::paint::Painters;
use gpui::{prelude::*, *};
use spaceport_client::protocol::{AgentStatus, ClientShellAgent, ClientShellSnapshot};
use spaceport_client::text::{caption, safe_text};
use spaceport_settings::herdr_settings::Settings;
use spaceport_settings::{AgentSort, FontConfig, StatusIcons, Theme};

/// Prepared, comparable props for the cached sidebar, never loaded during render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Indicators {
    /// The set every status is drawn with.
    pub icons: StatusIcons,
    /// Whether busy and finished rows carry a bar on their leading edge.
    pub edge_rail: bool,
    colors: [u32; 5],
    /// The opacity of the row a mark sits in, as `f32` bits: moving marks
    /// draw in the window's mark layer, outside the row's own opacity.
    fade: u32,
}

impl Indicators {
    /// Herdr's settings supply only the status colors; its indicator style
    /// is the terminal client's, and this app draws `icons` instead.
    pub fn new(
        settings: Option<&Settings>,
        icons: StatusIcons,
        edge_rail: bool,
        light: bool,
        theme: &Theme,
    ) -> Self {
        Self {
            icons,
            edge_rail,
            colors: [
                AgentStatus::Unknown,
                AgentStatus::Idle,
                AgentStatus::Working,
                AgentStatus::Done,
                AgentStatus::Blocked,
            ]
            .map(|status| {
                settings.map_or_else(
                    || status_style(status, theme).2,
                    |settings| theme.ink(settings.status_color(status, light)),
                )
            }),
            fade: 1f32.to_bits(),
        }
    }

    /// The same marks in a row drawn at `opacity`.
    pub fn faded(self, opacity: f32) -> Self {
        Self {
            fade: opacity.to_bits(),
            ..self
        }
    }

    pub fn fade(self) -> f32 {
        f32::from_bits(self.fade)
    }

    /// The same colors drawn with another set, as a preview of it.
    pub fn with_icons(self, icons: StatusIcons) -> Self {
        Self { icons, ..self }
    }

    pub fn color(self, status: AgentStatus) -> u32 {
        self.colors[match status {
            AgentStatus::Unknown => 0,
            AgentStatus::Idle => 1,
            AgentStatus::Working => 2,
            AgentStatus::Done => 3,
            AgentStatus::Blocked => 4,
        }]
    }

    pub fn width(self, font: &FontConfig) -> f32 {
        match self.icons {
            StatusIcons::Dots => scaled(font, STATUS_WIDTH),
            StatusIcons::Tiles => tile_size(font),
            _ => icon_size(font),
        }
    }

    /// Whether a row centers the mark across all its lines. Drawn icons are
    /// taller than a line and read as the row's status, not its first line's.
    pub fn centered(self) -> bool {
        self.icons != StatusIcons::Dots
    }
}

/// A drawn icon's side: as tall as a line fits, since compact rows are one
/// line with no padding to spill into, and even so its 16-unit grid lands on
/// whole device pixels at 1x and 2x.
pub fn icon_size(font: &FontConfig) -> f32 {
    (line_height(font) / 2.).floor().max(5.) * 2.
}

/// A tile is the same size as the other icons; its mark leaves it two pixels
/// of tint on every side.
pub fn tile_size(font: &FontConfig) -> f32 {
    icon_size(font)
}

/// The daemon's name for a status, which keys an integration's
/// `state_labels` and what sidebar rules on `state_text` match.
pub fn status_key(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "working", // i18n: data
        AgentStatus::Blocked => "blocked", // i18n: data
        AgentStatus::Done => "done",       // i18n: data
        AgentStatus::Idle => "idle",       // i18n: data
        AgentStatus::Unknown => "unknown", // i18n: data
    }
}

/// Herdr's own cap on a state label, so a longer one came from a peer that
/// ignored it and is cut where the terminal client would have cut it.
const STATE_LABEL_LIMIT: usize = 80;

/// The words an agent's integration chose for its current status, as Herdr's
/// `state_labels` metadata, else `fallback`. Labels are untrusted display
/// text: bounded and stripped of controls and bidi overrides.
pub fn state_label<'a>(agent: &ClientShellAgent, fallback: &'a str) -> std::borrow::Cow<'a, str> {
    let key = status_key(agent.agent_status);
    agent
        .state_labels
        .iter()
        .find(|(state, _)| state == key)
        .map(|(_, label)| safe_text(label, STATE_LABEL_LIMIT))
        .map(|label| label.trim().to_owned())
        .filter(|label| !label.is_empty())
        .map_or(
            std::borrow::Cow::Borrowed(fallback),
            std::borrow::Cow::Owned,
        )
}

/// The name an agent's row shows for its pane: the label set with Rename,
/// otherwise the title an unnamed pane already carries. Both are untrusted
/// daemon and terminal text, so the name is safe, bounded text, and none when
/// it is blank or only repeats the agent's `name`.
pub fn pane_name(
    agent: &ClientShellAgent,
    snapshot: &ClientShellSnapshot,
    name: &str,
) -> Option<String> {
    let renamed = snapshot
        .panes
        .iter()
        .find(|pane| pane.pane_id == agent.pane_id)
        .and_then(|pane| safe_caption(pane.label.as_deref()));
    renamed
        .or_else(|| safe_caption(agent.title.as_deref()))
        .filter(|pane| pane != name.trim())
}

/// Pane names stay far longer than any row can show, so the bound only stops
/// a runaway title from being scanned and laid out in full.
pub const PANE_CAPTION_LIMIT: usize = 160;

fn safe_caption(text: Option<&str>) -> Option<String> {
    let text = safe_text(caption(text?)?, PANE_CAPTION_LIMIT);
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// Where an agent runs: its workspace, and its tab when that earns a place,
/// which upstream decides by the workspace having several tabs or the user
/// naming it. `None` once the agent's workspace has gone.
pub fn agent_place<'a>(
    agent: &ClientShellAgent,
    snapshot: &'a ClientShellSnapshot,
) -> Option<(&'a str, Option<&'a str>)> {
    let workspace = snapshot
        .workspaces
        .iter()
        .find(|workspace| workspace.workspace_id == agent.workspace_id)?;
    let tabs = snapshot
        .tabs
        .iter()
        .filter(|tab| tab.workspace_id == agent.workspace_id)
        .count();
    let tab = snapshot
        .tabs
        .iter()
        .find(|tab| tab.tab_id == agent.tab_id)
        .filter(|tab| tabs > 1 || tab.custom_label)
        .and_then(|tab| caption(&tab.label));
    Some((workspace.label.as_str(), tab))
}

/// Upstream's default agent rows: host, workspace, tab, and pane name on the
/// first line, the agent itself on the second. A pane whose workspace has gone
/// leaves the agent to name the row, with the pane name beside it.
pub fn agent_labels<'a>(
    name: &'a str,
    place: Option<(&'a str, Option<&'a str>)>,
    host: Option<&'a str>,
    pane: Option<&'a str>,
) -> (Vec<(&'a str, bool)>, &'a str) {
    let Some((workspace, tab)) = place else {
        let segments = [Some((name, true)), pane.map(|pane| (pane, false))];
        return (segments.into_iter().flatten().collect(), "");
    };
    // Only the workspace carries the row's weight: upstream paints the host and
    // tab around it in its secondary color. The pane name joins them there.
    let segments = [
        (host, false),
        (Some(workspace), true),
        (tab, false),
        (pane, false),
    ]
    .into_iter()
    .filter_map(|(text, primary)| Some((text?, primary)))
    .filter(|(text, _)| !text.is_empty())
    .collect();
    (segments, name)
}

/// The agents of one endpoint in the order the panel paints them.
///
/// A plugin's agent view (`agent.view.set`) is authoritative while the daemon
/// names it: the panel shows exactly the panes in `agent_order`, in that
/// order, so agents its filter excluded stay hidden and an empty order means
/// none match. Upstream's client keys this on the label, not on the order
/// being non-empty, and drops ids the snapshot has no agent for. Without a
/// view the local sort applies.
pub fn sorted_agents(snapshot: &ClientShellSnapshot, sort: AgentSort) -> Vec<&ClientShellAgent> {
    if snapshot.agent_view_label.is_some() {
        return snapshot
            .agent_order
            .iter()
            .filter_map(|pane_id| {
                snapshot
                    .agents
                    .iter()
                    .find(|agent| agent.pane_id == *pane_id)
            })
            .collect();
    }
    let mut ordered: Vec<_> = snapshot.agents.iter().collect();
    if sort == AgentSort::Priority {
        ordered.sort_by_key(|agent| {
            (
                std::cmp::Reverse(status_priority(agent.agent_status)),
                std::cmp::Reverse(agent.state_change_seq),
            )
        });
    }
    ordered
}

/// Upstream draws status from its own palette, defaulting to Catppuccin Mocha,
/// and never from the terminal's ANSI colors. Matching those literals keeps a
/// dot the same color in both clients whatever terminal theme is loaded, where
/// ANSI slots would drift: Xcode Dark paints its cyan purple. Mocha's pastels
/// vanish on light chrome, so [`Theme::ink`] darkens them there, keeping hue.
/// The diameter is the default size's, which callers scale to their font.
pub fn status_style(status: AgentStatus, theme: &Theme) -> (f32, bool, u32) {
    let (diameter, filled, color) = match status {
        AgentStatus::Working => (STATUS_WIDTH, true, 0xf9e2af),
        AgentStatus::Blocked => (STATUS_WIDTH, true, 0xf38ba8),
        AgentStatus::Done => (STATUS_WIDTH, true, 0x94e2d5),
        AgentStatus::Idle => (STATUS_WIDTH, false, 0xa6e3a1),
        AgentStatus::Unknown => (STATUS_DOT_UNKNOWN, true, 0x6c7086),
    };
    (diameter, filled, theme.ink(color))
}

const RAIL_WIDTH: f32 = 3.;
/// The bar's inset from the row's top and bottom, as a share of its height,
/// so even a one-line Compact row at the smallest font keeps a visible bar.
const RAIL_INSET: f32 = 0.2;

/// The edge rail's bar on a row's leading edge, for the statuses worth
/// noticing. Absolutely placed, so the row must be `relative`, and mirrored
/// when the chrome runs right to left.
pub fn rail(
    indicators: Indicators,
    status: AgentStatus,
    key: &str,
    right_to_left: bool,
) -> Option<Div> {
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
            .inset_s(right_to_left, px(0.))
            .top(relative(RAIL_INSET))
            .bottom(relative(RAIL_INSET))
            .w(px(RAIL_WIDTH))
            .rounded_e(right_to_left, px(RAIL_WIDTH))
            .bg(rgb(indicators.color(status))),
    )
}

pub fn status_indicator(
    status: AgentStatus,
    font: &FontConfig,
    indicators: Indicators,
    painters: Painters,
    right_to_left: bool,
) -> Div {
    status_mark(
        status,
        font,
        indicators,
        indicators.color(status),
        false,
        painters,
        right_to_left,
    )
}

/// Configured tokens can override color and weight without changing the set;
/// only dots have a weight to change.
pub fn status_mark(
    status: AgentStatus,
    font: &FontConfig,
    indicators: Indicators,
    color: u32,
    bold: bool,
    painters: Painters,
    right_to_left: bool,
) -> Div {
    if indicators.icons != StatusIcons::Dots {
        return status_icon(status, font, indicators, color, painters, right_to_left);
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
        .row(right_to_left)
        .items_center()
        .justify_center()
        .when(status == AgentStatus::Working, |slot| {
            slot.child(icon_mark(
                painters,
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
        Animation::new(spaceport_motion::duration(
            std::time::Duration::from_millis(period_ms),
        ))
        .repeat_synced()
        .with_max_fps(spaceport_motion::MARK_MAX_FPS),
        |mark, delta| mark.with_transformation(Transformation::rotate(percentage(delta))),
    )
}

/// A drawn status icon, centered in its slot. Configured weights do not
/// apply: the icon's strokes are fixed by its asset.
fn status_icon(
    status: AgentStatus,
    font: &FontConfig,
    indicators: Indicators,
    color: u32,
    painters: Painters,
    right_to_left: bool,
) -> Div {
    let (icons, fade) = (indicators.icons, indicators.fade());
    let size = icon_size(font);
    let slot = div()
        .flex_none()
        .row(right_to_left)
        .items_center()
        .justify_center();
    if icons != StatusIcons::Tiles {
        return slot
            .size(px(size))
            .child(icon_mark(painters, icons, status, size, color, fade));
    }
    slot.size(px(tile_size(font)))
        .rounded(px(spaceport_settings::corners::SMALL))
        .bg(rgba((color << 8) | 0x30))
        .border_1()
        .border_color(rgba((color << 8) | 0x70))
        .child(icon_mark(painters, icons, status, size - 4., color, fade))
}

/// One set's mark for `status` at `size`: its asset, turning while working,
/// or the mark drawn for the sets without assets. The working assets are
/// rings around a turnable center, so spinning the whole icon keeps its
/// outline still. A mark that moves is drawn where the window can compose it
/// through `painters`, so its frames never rebuild the sidebar; `fade` is the
/// opacity of its row, which that layer cannot inherit.
pub fn icon_mark(
    painters: Painters,
    icons: StatusIcons,
    status: AgentStatus,
    size: f32,
    color: u32,
    fade: f32,
) -> AnyElement {
    // Working always moves; of the settled statuses only a drawn blocked
    // mark can.
    let moves = status == AgentStatus::Working
        || (status == AgentStatus::Blocked
            && spaceport_icons::status_icon(icons, status).is_none());
    if !moves {
        return drawn_mark(icons, status, size, color);
    }
    painters.layered(size, fade, move || drawn_mark(icons, status, size, color))
}

fn drawn_mark(icons: StatusIcons, status: AgentStatus, size: f32, color: u32) -> AnyElement {
    if icons == StatusIcons::Dots && status == AgentStatus::Working {
        return working_orbit(size, color).into_any_element();
    }
    let Some(path) = spaceport_icons::status_icon(icons, status) else {
        return crate::animated::mark(icons, status, size, color);
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

#[cfg(test)]
mod tests;
