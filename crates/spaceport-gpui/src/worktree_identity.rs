//! Linked Git worktrees told apart from their repository's main checkout.
//!
//! The daemon already reports, per workspace, whether its checkout is a linked
//! worktree (its `.git` is a file pointing into another repository's
//! `worktrees/`), so nothing here reads Git. In the sidebar a linked worktree
//! hangs from its main checkout on the tree's guide lines, in every layout,
//! and one standing alone (its main checkout not open) takes the branch glyph
//! instead, since it has no line to hang from. In the workspace, when
//! `apply_to` asks for `"terminal"`, a hairline runs where its tab strip
//! meets its terminals, and none while the strip is hidden, since it would
//! then cover cells; it is off by default. The main checkout keeps the
//! theme as drawn, so the odd one out is the copy, not the original.
//!
//! The guides are structure, not identity, so they keep the tree's own muted
//! stroke and stay quiet however many worktrees are open. The glyph and the
//! rule carry the hue: the theme's blue, as the notes kept on worktrees draw
//! it, never [`Theme::primary`], which already means "selected".

use crate::config::{Theme, Worktrees};
use gpui::{Div, ParentElement, Styled, div, px, rgb};
use spaceport_client::protocol::{ClientShellSnapshot, ClientShellWorkspace};

#[cfg(test)]
mod tests;

/// The hue as a mark on `theme`'s chrome: the glyph and the rule. Its
/// lightness moves only as far as the contrast setting needs to read on the
/// background, the surface, a selected row, and Herdr's own sidebar colour
/// when the theme sets one, so a configured colour keeps its hue.
pub(crate) fn mark(worktrees: &Worktrees, theme: &Theme) -> u32 {
    let mark = theme.ink(worktrees.tint.unwrap_or(theme.palette[BLUE]));
    match theme.sidebar {
        Some(sidebar) => crate::contrast::ink_on_chrome(
            mark,
            [sidebar, theme.surface, theme.active],
            theme.contrast.mark_ratio(),
        ),
        None => mark,
    }
}

/// The branch glyph's colour on `theme` for a linked worktree standing
/// alone, unless the sidebar does not show the marks.
pub(crate) fn glyph(worktrees: &Worktrees, theme: &Theme) -> Option<u32> {
    worktrees.sidebar.then(|| mark(worktrees, theme))
}

/// The rule along the edge of a linked worktree's terminals, when they show
/// one. Absolutely placed: the caller pins it to the edge, so the terminal
/// keeps its size whichever workspace is shown.
pub(crate) fn rule(worktrees: &Worktrees, linked: bool, theme: &Theme) -> Option<Div> {
    (linked && worktrees.terminal).then(|| {
        div()
            .absolute()
            .left_0()
            .right_0()
            .h(px(RULE))
            .bg(rgb(mark(worktrees, theme)))
    })
}

/// Thickness of the rule along a linked worktree's tab strip, in logical
/// pixels: as thin as reads at a glance without becoming a border.
pub(crate) const RULE: f32 = 2.;

/// The palette entry the default hue comes from: the theme's blue.
const BLUE: usize = 4;

/// Whether `workspace` is a linked worktree rather than a main checkout or a
/// folder outside Git, as the daemon reports it.
pub(crate) fn is_linked(workspace: &ClientShellWorkspace) -> bool {
    workspace
        .worktree
        .as_ref()
        .is_some_and(|tree| tree.is_linked_worktree)
}

/// Whether the workspace `snapshot` shows is a linked worktree.
pub(crate) fn focused_is_linked(snapshot: &ClientShellSnapshot) -> bool {
    snapshot
        .workspaces
        .iter()
        .find(|workspace| Some(&workspace.workspace_id) == snapshot.focused_workspace_id.as_ref())
        .is_some_and(is_linked)
}

/// A group's strip with its rule, along the strip's edge that meets the
/// terminals and inside the chrome so no cell is covered. `below` is a strip
/// under the terminals. A group that shows no strip has no chrome edge, and
/// a rule along its top would lie over the first row of cells, so it shows
/// none.
pub(crate) fn place_rule(strip: Option<Div>, rule: Option<Div>, below: bool) -> Option<Div> {
    let strip = strip?;
    Some(match rule {
        Some(rule) => strip.child(if below { rule.top_0() } else { rule.bottom_0() }),
        None => strip,
    })
}
