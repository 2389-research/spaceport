// ABOUTME: Captures only visible worktree labels, paths, search text, and scroll placement.
// ABOUTME: Excludes source targets, workspace IDs, subscriptions, and input entities from departure.
use super::*;

pub(super) struct Row {
    pub(super) index: usize,
    pub(super) label: String,
    pub(super) path: String,
    pub(super) status: String,
    pub(super) selected: bool,
}

pub(in crate::menu) struct Paint {
    pub(super) search: crate::search_input::Paint,
    pub(super) count: String,
    pub(super) empty: Option<(bool, String)>,
    pub(super) rows: Vec<Row>,
    pub(super) height: Pixels,
    pub(super) offset: Pixels,
}

pub(super) fn row(entry: &Entry, index: usize, selected: usize) -> Row {
    Row {
        index,
        label: entry.branch.as_ref().unwrap_or(&entry.label).clone(),
        path: entry.path.clone(),
        status: match (entry.open_workspace_id.is_some(), entry.is_detached) {
            (true, _) => fl!("worktree-open-open"),
            (_, true) => fl!("worktree-open-detached"),
            _ => String::new(),
        },
        selected: index == selected,
    }
}

impl SheprdWindow {
    pub(in crate::menu) fn existing_worktrees_paint(&self, cx: &App) -> Option<Paint> {
        let picker = self.menu.worktree_open.as_ref()?;
        let scroll = picker.scroll.0.borrow();
        let height = scroll
            .last_item_size
            .filter(|_| !picker.filtered.is_empty())
            .map_or(self.config.ui.line_height() * 2. + 14., |size| {
                f32::from(size.contents.height) / picker.filtered.len() as f32
            })
            .max(1.);
        let offset = scroll.base_handle.offset().y;
        let start =
            ((-f32::from(offset) / height).floor().max(0.) as usize).min(picker.filtered.len());
        let length = (f32::from(scroll.base_handle.bounds().size.height).max(0.) / height).ceil()
            as usize
            + 1;
        let end = start.saturating_add(length).min(picker.filtered.len());
        Some(Paint {
            search: picker.search.read(cx).paint(),
            count: count(picker),
            empty: self.worktree_empty(picker),
            rows: (start..end)
                .filter_map(|index| {
                    picker
                        .entry(index)
                        .map(|entry| row(entry, index, picker.selected))
                })
                .collect(),
            height: px(height),
            offset: offset + px(height * start as f32),
        })
    }

    pub(super) fn worktree_empty(&self, picker: &Picker) -> Option<(bool, String)> {
        picker.filtered.is_empty().then(|| {
            (
                picker.entries.is_empty(),
                if picker.pending.is_some() {
                    fl!("worktree-open-loading")
                } else if !picker.entries.is_empty() {
                    fl!("worktree-open-no-matching")
                } else if self.menu.error.is_some() {
                    fl!("worktree-open-unavailable")
                } else {
                    fl!("worktree-open-empty")
                },
            )
        })
    }
}

pub(super) fn count(picker: &Picker) -> String {
    fl!(
        "worktree-open-count",
        shown = picker.filtered.len(),
        total = picker.entries.len()
    )
}
