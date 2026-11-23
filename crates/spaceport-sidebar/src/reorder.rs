//! Reordering workspaces by dragging their rows. Holding a press on a row, or
//! moving it a few pixels, lifts the row; the rows around it shift as if it
//! had already been dropped under the pointer, so the gap they open is where
//! it lands. Releasing sends the move: a lone workspace as `workspace.move`,
//! as Herdr's own TUI sends an ungrouped row, and a worktree group as one
//! `workspace.move_block`. The daemon owns the order, so the preview holds
//! until the next snapshot reorders the list, the same for every attached
//! client.
//!
//! A top-level row carries its whole worktree group, collapsed children and
//! the group's other parents included, and lands between other top-level
//! rows. A linked worktree only moves among its own siblings: the group is
//! decided by its repository, not by where it is dropped.
//!
//! A drag's life is here, from its press to the preview it holds after the
//! drop. Spawning its timers, sending its move, and reading the connection it
//! is sent over stay in the app.

use crate::workspaces::{grouped_keys, workspace_entries};
use gpui::{Pixels, Point, ScrollHandle, Task};
use spaceport_client::{
    protocol::ClientShellWorkspace,
    workspaces::{WorkspaceMoveBlockParams, WorkspaceMoveParams, WorkspaceReorder},
};
use spaceport_reorder::{SLIDE, Slide, slot_for};
use std::{cell::RefCell, collections::HashMap, time::Instant};

/// A press on a workspace row that may become a reorder.
pub struct WorkspaceDrag {
    pub boot: String,
    pub workspace: String,
    pub origin: Point<Pixels>,
    pub pointer: Point<Pixels>,
    pub phase: Phase,
    /// Each shifted row's slide, by workspace. Render reads and advances them,
    /// so they live behind a cell; they end with the drag.
    slides: RefCell<HashMap<String, Slide>>,
    /// Lifts the row once the press has rested, then ends a dropped preview
    /// the daemon never answered. Dropping the drag cancels it.
    _timer: Task<()>,
}

/// Where a drag is in its life. A press lifts, and a lifted row dropped over
/// a gap settles; nothing goes back a step.
pub enum Phase {
    /// Pressed, and not yet held or moved far enough to lift.
    Pressed,
    /// Lifted and following the pointer. `target` is the gap under it, `None`
    /// over the carried unit's own place.
    Floating { target: Option<Target> },
    /// Dropped into `target` with its move sent. `order` is the order the
    /// list had then, kept so the preview lasts exactly until the daemon's
    /// answer.
    Dropped { target: Target, order: Vec<String> },
}

impl WorkspaceDrag {
    /// A press on `workspace` of `boot`, which `lift` raises once it has
    /// rested.
    pub fn press(boot: String, workspace: &str, position: Point<Pixels>, lift: Task<()>) -> Self {
        Self {
            boot,
            workspace: workspace.to_owned(),
            origin: position,
            pointer: position,
            phase: Phase::Pressed,
            slides: RefCell::default(),
            _timer: lift,
        }
    }

    /// The same drag dropped into `target`, keeping `order` so the preview
    /// holds until the daemon's answer, with `settle` to end it if none comes.
    pub fn dropped(self, target: Target, order: Vec<String>, settle: Task<()>) -> Self {
        Self {
            phase: Phase::Dropped { target, order },
            _timer: settle,
            ..self
        }
    }

    /// How far the lifted row has followed the pointer.
    pub fn offset(&self) -> Pixels {
        self.pointer.y - self.origin.y
    }

    /// Where a row paints on its way to shift `to`, and whether it is still
    /// moving. A row starts from its resting place.
    pub fn slide(&self, workspace: &str, to: f32, now: Instant) -> (f32, bool) {
        let mut slides = self.slides.borrow_mut();
        let slide = slides.entry(workspace.to_owned()).or_insert(Slide {
            from: 0.,
            to: 0.,
            start: now,
        });
        *slide = slide.toward(to, now, SLIDE);
        (slide.at(now, SLIDE), slide.progress(now, SLIDE) < 1.)
    }

    /// Records a carried row at the pointer, so it slides from there once
    /// dropped rather than from where it was picked up.
    pub fn pin(&self, workspace: &str, at: f32, now: Instant) {
        self.slides.borrow_mut().insert(
            workspace.to_owned(),
            Slide {
                from: at,
                to: at,
                start: now,
            },
        );
    }

    /// Whether the lifted row still follows the pointer.
    pub fn floating(&self) -> bool {
        matches!(self.phase, Phase::Floating { .. })
    }

    /// Whether the press has lifted, floating or dropped.
    #[cfg(any(test, feature = "test-support"))]
    pub fn lifted(&self) -> bool {
        !matches!(self.phase, Phase::Pressed)
    }

    /// Whether the row was dropped with its move sent, and the daemon's
    /// answer is still awaited.
    pub fn settling(&self) -> bool {
        matches!(self.phase, Phase::Dropped { .. })
    }

    /// The gap the row floats over or was dropped into.
    pub fn target(&self) -> Option<&Target> {
        match &self.phase {
            Phase::Pressed => None,
            Phase::Floating { target } => target.as_ref(),
            Phase::Dropped { target, .. } => Some(target),
        }
    }

    /// Whether the list should show the drop in place: while it floats, and
    /// after it is dropped until the daemon's order replaces the old one.
    pub fn previewing(&self, workspaces: &[ClientShellWorkspace]) -> bool {
        match &self.phase {
            Phase::Pressed => false,
            Phase::Floating { .. } => true,
            Phase::Dropped { order, .. } => order.iter().map(String::as_str).eq(workspaces
                .iter()
                .map(|workspace| workspace.workspace_id.as_str())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub slot: usize,
    pub request: MoveBlock,
}

#[cfg(any(test, feature = "test-support"))]
impl Target {
    pub fn params(&self) -> serde_json::Value {
        self.request.params()
    }
}

/// Where a drop lands: `workspace_ids` land, in that order, before
/// `before`, or at the end of the list without one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveBlock {
    workspace_ids: Vec<String>,
    before: Option<String>,
}

impl MoveBlock {
    #[cfg(any(test, feature = "test-support"))]
    fn params(&self) -> serde_json::Value {
        serde_json::json!({
            "workspace_ids": self.workspace_ids,
            "before_workspace_id": self.before,
        })
    }

    /// The request that makes this move against `workspaces`, the daemon's
    /// current order. A lone workspace goes as `workspace.move` when the
    /// daemon offers it (`single`), so it announces `workspace.moved` as the
    /// same drag in Herdr's TUI does. A group's members may sit apart in the
    /// daemon's order, which only `workspace.move_block` gathers in one step.
    pub fn reorder(&self, workspaces: &[ClientShellWorkspace], single: bool) -> WorkspaceReorder {
        if single && let [workspace_id] = self.workspace_ids.as_slice() {
            // The index counts the workspace still in place, so landing
            // before `before` is `before`'s own index, and last is the length.
            let insert_index = match &self.before {
                Some(before) => workspaces
                    .iter()
                    .position(|workspace| &workspace.workspace_id == before),
                None => Some(workspaces.len()),
            };
            if let Some(insert_index) = insert_index {
                return WorkspaceReorder::Move(WorkspaceMoveParams {
                    workspace_id: workspace_id.clone(),
                    insert_index,
                });
            }
        }
        WorkspaceReorder::MoveBlock(WorkspaceMoveBlockParams {
            workspace_ids: self.workspace_ids.clone(),
            before_workspace_id: self.before.clone(),
        })
    }
}

/// The units a drag reorders, in display order, and the one it carries. Each
/// unit lists workspace indices in the order the sidebar shows them.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    units: Vec<Vec<usize>>,
    dragged: usize,
    /// Whether the units are one group's children rather than top-level rows.
    children: bool,
}

impl Plan {
    /// `None` when the pressed row has nowhere else to go.
    pub fn new(workspaces: &[ClientShellWorkspace], workspace: &str) -> Option<Self> {
        let pressed = workspaces
            .iter()
            .position(|w| w.workspace_id == workspace)?;
        // A group's rows are contiguous, so a block is a run of one grouped
        // repository's rows, or a single ungrouped row.
        let grouped = grouped_keys(workspaces);
        let group_of = |index: usize| {
            workspaces[index]
                .worktree
                .as_ref()
                .map(|tree| tree.key.as_str())
                .filter(|key| grouped.contains(key))
        };
        let mut blocks: Vec<Vec<(usize, bool)>> = Vec::new();
        for (index, child) in workspace_entries(workspaces) {
            match blocks.last_mut() {
                Some(block)
                    if group_of(index).is_some() && group_of(block[0].0) == group_of(index) =>
                {
                    block.push((index, child));
                }
                _ => blocks.push(vec![(index, child)]),
            }
        }
        let (block, &(_, child)) = blocks.iter().enumerate().find_map(|(position, block)| {
            Some((
                position,
                block.iter().find(|&&(index, _)| index == pressed)?,
            ))
        })?;
        let plan = if !child {
            Self {
                units: blocks
                    .iter()
                    .map(|block| block.iter().map(|&(index, _)| index).collect())
                    .collect(),
                dragged: block,
                children: false,
            }
        } else {
            let children: Vec<_> = blocks[block]
                .iter()
                .filter(|&&(_, child)| child)
                .map(|&(i, _)| vec![i])
                .collect();
            Self {
                dragged: children.iter().position(|unit| unit[0] == pressed)?,
                units: children,
                children: true,
            }
        };
        (plan.units.len() > 1).then_some(plan)
    }

    /// How many units the drag reorders, and so how many gaps plus one.
    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    pub fn dragged(&self) -> usize {
        self.dragged
    }

    /// The unit a workspace belongs to, if this drag reorders it.
    pub fn unit_of(&self, index: usize) -> Option<usize> {
        self.units.iter().position(|unit| unit.contains(&index))
    }

    /// The move that dropping into `slot`, the gap before unit `slot`, makes.
    /// `None` for the two gaps around the carried unit, which move nothing.
    pub fn request(&self, workspaces: &[ClientShellWorkspace], slot: usize) -> Option<MoveBlock> {
        if slot == self.dragged || slot == self.dragged + 1 || slot > self.units.len() {
            return None;
        }
        let moved = &self.units[self.dragged];
        let id = |index: usize| workspaces[index].workspace_id.clone();
        // A group shows where its earliest member sits, so landing before a
        // unit means landing before that member.
        let before = match self.units.get(slot) {
            Some(unit) => unit.iter().min().copied(),
            // Past the last sibling, a child still has to stay ahead of what
            // follows its group; a top-level unit simply goes last.
            None if self.children => {
                let last = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|&(unit, _)| unit != self.dragged)
                    .flat_map(|(_, unit)| unit.iter().copied())
                    .max()?;
                (last + 1..workspaces.len()).find(|index| !moved.contains(index))
            }
            None => None,
        };
        Some(MoveBlock {
            workspace_ids: moved.iter().map(|&index| id(index)).collect(),
            before: before.map(id),
        })
    }
}

/// Resolves the pointer to a gap, from each unit's rows and the move for every
/// gap, all prepared by the render the pointer is over.
pub fn resolve(
    rows: &[(usize, usize, Pixels)],
    requests: &[Option<MoveBlock>],
    dragged: usize,
    scroll: &ScrollHandle,
    lift: Pixels,
) -> Option<Target> {
    let len = requests.len().checked_sub(1)?;
    let offset = scroll.offset().y;
    let mut spans: Vec<Option<(Pixels, Pixels)>> = vec![None; len];
    // Rows sit where the preview shifted them; the gaps are measured where
    // they rest, or the list would chase its own preview.
    for &(unit, row, shift) in rows {
        let Some(bounds) = scroll.bounds_for_item(row) else {
            continue;
        };
        let (top, bottom) = (
            bounds.top() + offset - shift,
            bounds.bottom() + offset - shift,
        );
        let span = &mut spans[unit];
        *span = Some(span.map_or((top, bottom), |(t, b)| (t.min(top), b.max(bottom))));
    }
    let spans: Vec<_> = spans
        .iter()
        .map(|span| span.map(|(top, bottom)| (f32::from(top), f32::from(bottom))))
        .collect();
    let (top, bottom) = (*spans.get(dragged)?)?;
    let lift = f32::from(lift);
    let slot = slot_for(&spans, dragged, (top + lift, bottom + lift));
    let request = requests.get(slot)?.clone()?;
    Some(Target { slot, request })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
