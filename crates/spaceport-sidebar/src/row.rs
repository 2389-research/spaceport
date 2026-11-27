//! What a sidebar row is before it is painted: which of the two kinds it is,
//! where it sits in its worktree group, what a drag is doing to it, the inputs
//! every row of one render shares, and the name weight and colors its two
//! lines take. The app paints every row, so the decisions a layout and the
//! painter must agree on live here, together with the badge a row's trailing
//! edge shows, which needs only a font and a theme.

mod badge;
mod configured;
mod paint;

pub use badge::RowBadge;
pub(crate) use configured::agent_line;
pub use configured::{
    TokenLook, configured_status, configured_status_style, leading_status, styled, token_column,
};
pub use paint::row;

use crate::{
    agents::Indicators,
    direction::Logical,
    layout::SidebarLook,
    metrics::{STATUS_WIDTH, line_height, scaled},
    paint::Painters,
    wash::HostMark,
};
use gpui::{prelude::*, *};
use spaceport_icons::AgentIcon;
use spaceport_settings::{FontConfig, Theme};

/// What a row lists, which decides how its two lines are weighted: upstream
/// keeps agent names bold throughout and reserves bold workspaces for the
/// current one, with the branch picking up the accent while it is focused.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Workspace,
    Agent(AgentIcon),
}

/// Name color, name weight, and detail color for a row.
pub fn row_text(kind: RowKind, focused: bool, theme: &Theme) -> (u32, FontWeight, u32) {
    let weight = if focused || matches!(kind, RowKind::Agent(_)) {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    };
    let name = if focused {
        theme.foreground
    } else {
        theme.subtext()
    };
    let detail = if focused && kind == RowKind::Workspace {
        theme.primary()
    } else {
        theme.muted
    };
    (name, weight, detail)
}

/// Where a row stands while a workspace is dragged: rows the lifted card
/// passes over stop answering hover, so only the drop line marks a place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowLift {
    #[default]
    Resting,
    Passed,
    Lifted,
}

/// How a row is marked. Hovering needs no state: layouts style it through
/// the row's hover group, so it costs nothing per frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowState {
    /// Agent rows use a full-width selection fill in every layout.
    pub agent: bool,
    /// The focused workspace or agent.
    pub selected: bool,
    /// Marked as if hovered while the pointer is elsewhere, such as the row a
    /// menu was opened for.
    pub highlighted: bool,
    /// Whether a workspace is being dragged, and whether this is the row it
    /// carries. Rows the carried one passes stop answering hover; the carried
    /// row floats over them, so it must paint an opaque, lifted surface.
    pub lift: RowLift,
}

/// Where a row sits in its worktree group, which decides whether the gutter
/// carries a trunk through the row or ends in an elbow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowTree {
    None,
    Child,
    LastChild,
}

/// Where a guide's tick meets its row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elbow {
    /// This far below the row's top: the middle of its first line.
    At(f32),
    /// Halfway down the row, for a row that centres its one line.
    Middle,
}

/// The first of `values` that says something, else `fallback`.
pub fn first_text<'a>(
    values: impl IntoIterator<Item = Option<&'a str>>,
    fallback: &'a str,
) -> &'a str {
    values
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|s| !s.is_empty())
        .unwrap_or(fallback)
}

/// What a row shows in its leading icon slot: a neutral repository icon, an
/// empty slot for a folder outside a checkout, or no slot for a child row or
/// a disabled icon setting. A linked worktree standing alone shows its branch
/// glyph so it never relies on colour alone.
pub enum RowIcon {
    None,
    Blank,
    Repository,
    Worktree(u32),
}

impl RowIcon {
    /// Whether the slot shows something; a blank one only keeps the room.
    pub fn draws(&self) -> bool {
        matches!(self, Self::Repository | Self::Worktree(_))
    }

    /// The icon's place on the leading edge of the line it opens, or nothing
    /// for a row without one.
    pub fn slot(self, key: &str, cx: &RowContext<'_>) -> Div {
        let probe = match self {
            Self::Worktree(_) => "worktree-icon",
            _ => "workspace-icon",
        };
        let size = scaled(cx.font, 12.);
        let slot = div()
            .debug_selector(|| format!("{probe}-{key}"))
            .absolute()
            .top(px((line_height(cx.font) - size) / 2.))
            .size(px(size))
            .flex_none()
            .overflow_hidden();
        let slot = slot.inset_s(cx.right_to_left, px(0.));
        slot.child(self.element(cx.theme.muted))
    }

    /// The icon filling its parent, or nothing for rows without one.
    pub fn element(self, color: u32) -> AnyElement {
        match self {
            Self::None | Self::Blank => Empty.into_any_element(),
            Self::Repository => repository_icon(color).size_full().into_any_element(),
            Self::Worktree(hue) => svg()
                .path("icons/git-branch.svg")
                .size_full()
                .text_color(rgb(hue))
                .into_any_element(),
        }
    }
}

/// A vector folder remains legible at the small sidebar size.
fn repository_icon(color: u32) -> Svg {
    svg()
        .path("icons/folder.svg")
        .flex_none()
        .text_color(rgb(color))
}

/// Trunk and tick for a child row, given the gutter the row reserves between
/// the parent's label column and the child's leading mark, with the tick at
/// `middle`, the middle of the row's first line. Snapped to whole device
/// pixels and painted as quads rather than borders: a bordered box rounds
/// each edge on its own, which left the trunk thinner than its tick.
/// `right_to_left` puts the leading edge on the gutter's right.
pub fn tree_lines(
    gutter: Bounds<Pixels>,
    tree: RowTree,
    middle: Pixels,
    scale: f32,
    right_to_left: bool,
) -> [Bounds<Pixels>; 2] {
    let device = |value: Pixels| f32::from(value) * scale;
    let logical = |value: f32| px(value / scale);
    let weight = scale.round().max(1.);
    let snap = |value: Pixels| logical(device(value).round());
    // The trunk runs down the gutter's leading edge; the tick crosses it at the
    // row's first line and stops where the row's leading mark begins.
    let x = snap(gutter.origin.x);
    let middle = logical((device(middle) - weight / 2.).round());
    let end = if tree == RowTree::LastChild {
        middle + logical(weight)
    } else {
        snap(gutter.bottom())
    };
    let lines = [
        Bounds::from_corners(
            point(x, snap(gutter.origin.y)),
            point(x + logical(weight), end),
        ),
        Bounds::from_corners(
            point(x, middle),
            point(snap(gutter.right()), middle + logical(weight)),
        ),
    ];
    // Right to left, the leading edge is the gutter's right one.
    if !right_to_left {
        return lines;
    }
    lines.map(|line| {
        let left = snap(gutter.origin.x + gutter.right() - line.right());
        Bounds::new(point(left, line.origin.y), line.size)
    })
}

/// Read-only inputs every row of one render shares.
pub struct RowContext<'a> {
    /// The application icon identifies the agent in a narrow row.
    pub icon_only_agent: bool,
    pub indicators: Indicators,
    pub font: &'a FontConfig,
    pub theme: &'a Theme,
    /// Density and style: spacing, highlight shape, and which details show.
    pub look: SidebarLook,
    /// Full sidebar width, divider included.
    pub width: f32,
    /// How far workspace rows step in under their host's header: zero while
    /// a single host shows no header to nest under.
    pub nest: f32,
    /// The host's colour wash and dimming, shared by every row it owns.
    pub mark: HostMark,
    /// The host these rows live on, named only while several hosts are
    /// listed and this one is remote, so a single-host sidebar stays quiet.
    pub host: Option<&'a str>,
    /// Whether the chrome lays out right to left, which mirrors the side a
    /// row's leading edge is on. The app reads the shown locale's direction;
    /// a crate cannot, so it is an input like the rest.
    pub right_to_left: bool,
    /// How the row reaches what only the app can paint: the element its text
    /// is, the window's layer for a mark that moves, and the copy for a word.
    pub painters: Painters,
}

impl RowContext<'_> {
    /// The indicators for a status mark in one of these rows: faded with the
    /// row's own fade and the host's dimming, which a mark drawn in the
    /// window's mark layer does not inherit.
    pub fn status_indicators(&self) -> Indicators {
        self.indicators
            .faded(self.indicators.fade() * self.mark.opacity())
    }
}

/// The gutter a child of Herdr's rows reserves for its guide, as `(x,
/// width)` from the sidebar's leading edge: from the parent's label column
/// to the child's status dot. A child steps in past a status slot wider than
/// the default dot, as the parent's label does, so the width does not depend
/// on the font or the indicator set; Minimal's rows nest the same way. A
/// configured indent or gap can leave none.
pub fn tree_gutter(cx: &RowContext<'_>) -> (f32, f32) {
    let layout = cx.look.density;
    let extra_status_width = cx.indicators.width(cx.font) - STATUS_WIDTH;
    (
        cx.nest + cx.look.tree_gutter() + extra_status_width,
        layout.padding() + layout.child_indent() - layout.tree_gutter(),
    )
}
/// A worktree child's guide: a trunk down the leading edge of `gutter`
/// (`x` and width, in the row's own coordinates) and a tick across it at
/// `elbow`, ending where the row's leading mark begins. A last child's
/// trunk stops at its tick. Every layout draws its guides with this, in the
/// tree's muted colour and one device pixel's weight, so they read as one
/// family. None for a parent, while the look draws no guides, or when the
/// gutter has no room. Absolute: the row must be relative.
pub fn guide(
    key: &str,
    tree: RowTree,
    (x, width): (f32, f32),
    elbow: Elbow,
    cx: &RowContext<'_>,
) -> Option<Div> {
    if tree == RowTree::None || !cx.look.density.tree_lines || width <= 0. {
        return None;
    }
    let color = cx.theme.muted;
    let right_to_left = cx.right_to_left;
    let key = key.to_owned();
    let gutter = div()
        .debug_selector(move || format!("tree-{key}"))
        .absolute()
        .w(px(width))
        .top_0()
        .bottom_0()
        .inset_s(right_to_left, px(x));
    Some(
        gutter.child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let middle = match elbow {
                        Elbow::At(depth) => bounds.origin.y + px(depth),
                        Elbow::Middle => bounds.center().y,
                    };
                    for line in
                        tree_lines(bounds, tree, middle, window.scale_factor(), right_to_left)
                    {
                        window.paint_quad(fill(line, rgb(color)));
                    }
                },
            )
            .size_full(),
        ),
    )
}

/// The trunk a child row's guide carries on through the lines that ride
/// under it (its note, its ports), so the next sibling stays tied to the same
/// tree. `gutter` is the row's guide gutter, from the sidebar's leading edge.
/// A last child's elbow has already ended, so it has none, and neither does
/// a row whose look draws no guides. Absolute: the caller places it in a
/// relative box spanning those lines.
pub fn trunk_below(
    key: &str,
    tree: RowTree,
    (x, width): (f32, f32),
    cx: &RowContext<'_>,
) -> Option<Div> {
    if tree != RowTree::Child || !cx.look.density.tree_lines || width <= 0. {
        return None;
    }
    let color = cx.theme.muted;
    let right_to_left = cx.right_to_left;
    let key = key.to_owned();
    let trunk = div()
        .debug_selector(move || format!("trunk-{key}"))
        .absolute()
        .w(px(width))
        .top_0()
        .bottom_0()
        .inset_s(right_to_left, px(x));
    Some(
        trunk.child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let scale = window.scale_factor();
                    let snap = |value: Pixels| px((f32::from(value) * scale).round() / scale);
                    let weight = px(scale.round().max(1.) / scale);
                    let x = match right_to_left {
                        false => snap(bounds.origin.x),
                        true => snap(bounds.right()) - weight,
                    };
                    let trunk = Bounds::from_corners(
                        point(x, snap(bounds.origin.y)),
                        point(x + weight, snap(bounds.bottom())),
                    );
                    window.paint_quad(fill(trunk, rgb(color)));
                },
            )
            .size_full(),
        ),
    )
}

/// The pulse a status dot gives way to while its checkout or device is
/// removed, as large as the dot at `font`.
pub fn removing_dot(selector: &'static str, font: &FontConfig, theme: &Theme) -> Div {
    div()
        .debug_selector(move || selector.into())
        .size(px(scaled(font, STATUS_WIDTH)))
        .flex_none()
        .child(
            div()
                .size_full()
                .rounded_full()
                .bg(rgb(theme.primary()))
                .with_animation(
                    SharedString::from(format!("{selector}-pulse")),
                    Animation::new(spaceport_motion::duration(std::time::Duration::from_secs(
                        1,
                    )))
                    .repeat(),
                    |dot, delta| dot.opacity(0.3 + 0.7 * (delta * std::f32::consts::PI).sin()),
                ),
        )
}

/// The room an agent's own mark takes before the name or place beside it.
const AGENT_ICON_RESERVE: f32 = 16.;

/// An agent's icon and the room it takes before the name beside it.
pub fn agent_icon_size(font: &FontConfig) -> (f32, f32) {
    (scaled(font, 12.), scaled(font, AGENT_ICON_RESERVE))
}

/// The agent's icon on the leading edge of the line that names it, or beside
/// its place in a detail line.
pub fn agent_mark(key: &str, icon: AgentIcon, size: f32, color: u32, cx: &RowContext<'_>) -> Div {
    let mark = div()
        .debug_selector(|| format!("agent-icon-{key}"))
        .absolute()
        .top(px((line_height(cx.font) - size) / 2.))
        .size(px(size))
        .inset_s(cx.right_to_left, px(0.));
    mark.child(icon.sidebar_element(size, color, cx.theme))
}

#[cfg(test)]
mod tests;
