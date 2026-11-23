//! What a sidebar row is before it is painted: which of the two kinds it is,
//! where it sits in its worktree group, what a drag is doing to it, the inputs
//! every row of one render shares, and the name weight and colors its two
//! lines take. The app paints every row, so the decisions a layout and the
//! painter must agree on live here, together with the badge a row's trailing
//! edge shows, which needs only a font and a theme.

mod badge;

pub use badge::RowBadge;

use crate::{
    agents::Indicators,
    layout::SidebarLook,
    metrics::{STATUS_WIDTH, scaled},
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
#[derive(Clone, Copy, PartialEq, Eq)]
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
/// Read-only inputs every row of one render shares.
pub struct RowContext<'a> {
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
