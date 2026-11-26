//! Sidebar rows as swappable layouts.
//!
//! Render prepares what a row shows once, as typed data, and hands it to the
//! configured [`RowLayout`] through a [`Cell`]. The layout decides which of
//! those pieces appear and where: one can name the host and branch, another
//! only the name. The cell carries the row's state, so callers write
//! `cell.selected(focused).highlighted(menu_open).row()` and each layout
//! paints those states its own way.
//!
//! Rows are built and dropped within one render of the cached sidebar view, so
//! everything here borrows from the render rather than copying into owned
//! state: a layout costs one virtual call per row.

use crate::{
    direction::glyph,
    row::{RowBadge, RowContext, RowIcon, RowLift, RowState, RowTree},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::{AgentStatus, ClientShellWorkspace};
use spaceport_icons::AgentIcon;
use std::borrow::Cow;

/// A repository's fold control. The layout draws it; the app owns what
/// clicking it changes.
pub struct Fold {
    pub id: ElementId,
    /// Position of the row in its host's workspace list, for probes.
    pub index: usize,
    pub collapsed: bool,
    pub toggle: Box<ClickHandler>,
}

impl Fold {
    /// The fold chevron, sized by the caller, in the row's own direction.
    /// Clicking it never selects the row it sits on.
    pub fn element(self, cx: &RowContext<'_>) -> Stateful<Div> {
        let Self {
            id,
            index,
            collapsed,
            toggle,
        } = self;
        let (muted, foreground) = (cx.theme.muted, cx.theme.foreground);
        let chevron = match collapsed {
            true => glyph("\u{25b8}", cx.right_to_left),
            false => "\u{25be}",
        };
        div()
            .id(id)
            .debug_selector(move || format!("collapse-{index}"))
            .flex_none()
            .text_color(rgb(muted))
            .hover(move |style| style.text_color(rgb(foreground)))
            .cursor_pointer()
            .child(cx.painters.label(chevron))
            .on_click(move |event, window, cx| {
                cx.stop_propagation();
                toggle(event, window, cx);
            })
    }
}

/// A click listener, erased so rows of every group share one type.
pub type ClickHandler = dyn Fn(&ClickEvent, &mut Window, &mut App);

pub struct WorkspaceRow<'a> {
    pub workspace: &'a ClientShellWorkspace,
    /// What the row is called: a child's branch unless the user renamed it.
    pub label: &'a str,
    pub tree: RowTree,
    pub icon: RowIcon,
    pub fold: Option<Fold>,
    /// Whether the row belongs to a worktree group, which lines its badges up
    /// with the group's fold control.
    pub grouped: bool,
    pub badge: Option<RowBadge>,
    /// The checkout is being deleted.
    pub removing: bool,
    /// Status the row shows. A collapsed group's head takes its most urgent member.
    pub status: AgentStatus,
    /// Daemon token rows, painted in the layout's own frame. Empty keeps its native lines.
    pub lines: Vec<Vec<crate::tokens::ResolvedToken>>,
}

impl<'a> WorkspaceRow<'a> {
    /// The checked-out branch, if Git reported a non-blank one.
    pub fn branch(&self) -> Option<&'a str> {
        self.workspace
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|branch| !branch.is_empty())
    }

    pub fn status(&self) -> AgentStatus {
        self.status
    }
}

pub struct AgentRow<'a> {
    /// Stable probe key, unique across both lists.
    pub key: String,
    pub name: &'a str,
    pub icon: AgentIcon,
    pub status: AgentStatus,
    /// Its workspace and, when that earns a place, its tab. Missing once the
    /// workspace has gone.
    pub place: Option<(&'a str, Option<&'a str>)>,
    /// Its pane's name: Rename's label, else the pane's title, and never the
    /// agent name again. Safe, bounded text.
    pub pane: Option<String>,
    /// The daemon's `state_text` word when its sidebar config asks for it, so
    /// the GUI and the TUI name the status the same way: the agent's own
    /// state label when its integration set one.
    pub status_text: Option<Cow<'a, str>>,
    /// Daemon token rows, painted in the layout's own frame. Empty keeps its native lines.
    pub lines: Vec<Vec<crate::tokens::ResolvedToken>>,
}

/// What a row shows.
pub enum RowData<'a> {
    Workspace(WorkspaceRow<'a>),
    Agent(AgentRow<'a>),
}

/// Arranges a row's content and marks its state. Implementations are
/// stateless and read only what they are given, so render stays free of I/O
/// and allocation beyond the elements themselves. How a dragged row looks is
/// part of the layout too: `state.lift` says which row is carried.
pub trait RowLayout {
    fn workspace(&self, row: WorkspaceRow<'_>, state: RowState, cx: &RowContext<'_>) -> Div;
    fn agent(&self, row: AgentRow<'_>, state: RowState, cx: &RowContext<'_>) -> Div;
    /// What carries a worktree child's tree guide on through the lines under
    /// its row (note, ports) to the next sibling. Only layouts that draw
    /// guides have one to continue.
    fn trunk_below(&self, _key: &str, _tree: RowTree, _cx: &RowContext<'_>) -> Option<Div> {
        None
    }
}

/// One row waiting for its state: build it, mark it, then take the element.
#[must_use = "a cell draws nothing until `row` takes its element"]
pub struct Cell<'a> {
    layout: &'a dyn RowLayout,
    data: RowData<'a>,
    state: RowState,
    cx: &'a RowContext<'a>,
}

impl<'a> Cell<'a> {
    pub fn new(layout: &'a dyn RowLayout, data: RowData<'a>, cx: &'a RowContext<'a>) -> Self {
        Self {
            layout,
            data,
            state: RowState::default(),
            cx,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.state.selected = selected;
        self
    }

    pub fn highlighted(mut self, highlighted: bool) -> Self {
        self.state.highlighted = highlighted;
        self
    }

    pub fn lift(mut self, lift: RowLift) -> Self {
        self.state.lift = lift;
        self
    }

    /// The row element, laid out for the state the cell was given. The rail
    /// style's bar is added here, on the row's leading edge, so every layout
    /// shows it in the same place without drawing it itself.
    pub fn row(self) -> Div {
        let (row, rail) = match self.data {
            RowData::Workspace(row) => {
                let rail = (!row.removing)
                    .then(|| {
                        crate::agents::rail(
                            self.cx.indicators,
                            row.status,
                            row.label,
                            self.cx.right_to_left,
                        )
                    })
                    .flatten();
                (self.layout.workspace(row, self.state, self.cx), rail)
            }
            RowData::Agent(row) => {
                let rail = crate::agents::rail(
                    self.cx.indicators,
                    row.status,
                    &row.key,
                    self.cx.right_to_left,
                );
                let row = self
                    .layout
                    .agent(row, self.state, self.cx)
                    .when(self.state.selected, |row| {
                        row.child(self.cx.look.agent_outline(self.cx.theme))
                    });
                (row, rail)
            }
        };
        match rail {
            Some(rail) => row.relative().child(rail),
            None => row,
        }
    }
}
