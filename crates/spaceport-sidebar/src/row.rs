//! What a sidebar row is before it is painted: which of the two kinds it is,
//! where it sits in its worktree group, what a drag is doing to it, and the
//! name weight and colors its two lines take. The app draws every row, so the
//! decisions a layout and the painter must agree on live here.

use gpui::FontWeight;
use spaceport_icons::AgentIcon;
use spaceport_settings::Theme;

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
