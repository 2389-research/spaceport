// ABOUTME: Captures workspace popup display data before active targets reset.
// ABOUTME: Retains bounded drafts and visible worktree rows without request or reader ownership.
use super::*;
use popover::{Label, Tile, layout};

pub(super) struct ItemPaint {
    pub(super) action: WorkspaceMenuAction,
    pub(super) label: Label,
    pub(super) text: String,
    pub(super) spoken: String,
    pub(super) caption: String,
    pub(super) selected: bool,
    pub(super) enabled: bool,
}

impl ItemPaint {
    pub(super) fn new(tile: Tile, selected: Option<WorkspaceMenuAction>) -> Self {
        Self {
            action: tile.action,
            label: tile.label,
            text: tile.label.text(),
            spoken: tile.label.spoken(),
            caption: tile.caption(),
            selected: tile.enabled && selected == Some(tile.action),
            enabled: tile.enabled,
        }
    }
}

pub(in crate::menu) struct PopoverPaint {
    pub(super) offset: Pixels,
    pub(super) label: String,
    pub(super) branch: Option<String>,
    pub(super) mute: Option<ItemPaint>,
    pub(super) tiles: Vec<ItemPaint>,
    pub(super) rows: Vec<ItemPaint>,
}

pub(in crate::menu) struct DialogPaint {
    pub(super) offset: Pixels,
    pub(super) status_offset: Pixels,
    pub(super) action: WorkspaceAction,
    pub(super) label: String,
    pub(super) error: Option<String>,
    pub(super) creating: bool,
    pub(super) armed: bool,
    pub(super) input: Option<DialogInput>,
    pub(super) picker: Option<super::super::worktree_open::Paint>,
}

impl SheprdWindow {
    pub(in crate::menu) fn workspace_popover_paint(&self) -> Option<PopoverPaint> {
        let target = self.menu.target.as_ref()?;
        let items = self.workspace_items();
        let (tiles, rows) = layout(&items);
        let item = |action, label| {
            ItemPaint::new(
                Tile {
                    action,
                    label,
                    enabled: true,
                },
                self.menu.workspace_selected,
            )
        };
        Some(PopoverPaint {
            offset: self.menu.workspace_scroll.offset().y,
            label: target.label.clone(),
            branch: target
                .branch
                .clone()
                .filter(|branch| !branch.trim().is_empty()),
            mute: items
                .iter()
                .find(|(action, _)| action.in_header())
                .map(|&(action, label)| item(action, label)),
            tiles: tiles
                .into_iter()
                .map(|tile| ItemPaint::new(tile, self.menu.workspace_selected))
                .collect(),
            rows: rows
                .into_iter()
                .map(|(action, label)| item(action, label))
                .collect(),
        })
    }

    pub(in crate::menu) fn workspace_dialog_paint(&mut self, cx: &App) -> Option<DialogPaint> {
        let Page::Dialog(action) = self.menu.page? else {
            return None;
        };
        let mut paint = self.prepare_workspace_dialog(action, cx)?;
        paint.input = self.menu.input.take();
        paint.picker = (action == WorkspaceAction::OpenWorktree)
            .then(|| self.existing_worktrees_paint(cx))
            .flatten();
        Some(paint)
    }

    pub(super) fn prepare_workspace_dialog(
        &self,
        action: WorkspaceAction,
        cx: &App,
    ) -> Option<DialogPaint> {
        let creating = self.menu.opening.is_some();
        let armed = (action != WorkspaceAction::OpenWorktree
            || self.menu.worktree_open.as_ref().is_some_and(|picker| {
                picker.pending.is_none()
                    && !picker.filtered.is_empty()
                    && !picker.search.read(cx).is_composing()
            }))
            && !creating;
        Some(DialogPaint {
            offset: if action == WorkspaceAction::OpenWorktree {
                px(0.)
            } else {
                self.menu.workspace_scroll.offset().y
            },
            status_offset: if action == WorkspaceAction::OpenWorktree {
                self.menu
                    .worktree_open
                    .as_ref()
                    .map_or(px(0.), |picker| picker.status_scroll.offset().y)
            } else {
                px(0.)
            },
            action,
            label: self.menu.target.as_ref()?.label.clone(),
            error: self.menu.error.clone(),
            creating,
            armed,
            input: None,
            picker: None,
        })
    }
}
