// ABOUTME: Loads bounded existing-worktree listings from the selected daemon.
// ABOUTME: Keeps request ownership separate from the picker display projection.
//! Existing checkouts come only from the selected daemon, never local Git.

// i18n: migrated
use super::{MenuState, Page, WorkspaceAction, refusal};
use crate::{SheprdWindow, i18n::fl, search_input::SearchInput};
use gpui::{prelude::*, *};
use serde::Deserialize;
use sheprd_client::Method;

mod paint;
mod render;
pub(in crate::menu) use paint::Paint;

const MAX_ENTRIES: usize = 512;

#[derive(Deserialize)]
pub(super) struct Source {
    pub(super) repo_key: String,
    repo_name: String,
    source_workspace_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct Entry {
    pub(super) path: String,
    pub(super) branch: Option<String>,
    pub(super) label: String,
    is_bare: bool,
    is_prunable: bool,
    pub(super) is_detached: bool,
    pub(super) open_workspace_id: Option<String>,
}

impl Entry {
    /// Whether a lowercased search matches the checkout's path, label or branch.
    pub(super) fn matches(&self, query: &str) -> bool {
        [&self.path, &self.label]
            .into_iter()
            .map(String::as_str)
            .chain(self.branch.as_deref())
            .any(|text| text.to_lowercase().contains(query))
    }
}

pub(super) struct Picker {
    pub(super) search: Entity<SearchInput>,
    _subscription: Option<Subscription>,
    pub(super) pending: Option<String>,
    pub(super) entries: Vec<Entry>,
    pub(super) source: Option<Source>,
    pub(super) filtered: Vec<usize>,
    query: String,
    pub(super) selected: usize,
    pub(super) scroll: UniformListScrollHandle,
    pub(super) status_scroll: ScrollHandle,
}

impl Picker {
    pub(super) fn new(search: Entity<SearchInput>) -> Self {
        Self {
            search,
            _subscription: None,
            pending: None,
            entries: Vec::new(),
            source: None,
            filtered: Vec::new(),
            query: String::new(),
            selected: 0,
            scroll: UniformListScrollHandle::new(),
            status_scroll: ScrollHandle::new(),
        }
    }

    pub(super) fn filter(&mut self, query: &str) {
        self.query = query.trim().to_lowercase();
        self.filtered = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.matches(&self.query))
            .map(|(index, _)| index)
            .collect();
        self.selected = 0;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
    }

    pub(super) fn entry(&self, row: usize) -> Option<&Entry> {
        self.entries.get(*self.filtered.get(row)?)
    }
}

pub(super) fn listing(response: &serde_json::Value) -> crate::Result<(Source, Vec<Entry>)> {
    let result = &response["result"];
    let rows = result["worktrees"]
        .as_array()
        .filter(|rows| result["type"] == "worktree_list" && rows.len() <= MAX_ENTRIES)
        .ok_or(crate::Error::WorktreeList)?;
    for field in ["repo_key", "repo_name", "source_workspace_id"] {
        if result["source"][field]
            .as_str()
            .is_some_and(|text| text.is_empty() || text.len() > 8192 || text.contains('\0'))
        {
            return Err(crate::Error::WorktreeList);
        }
    }
    let source =
        Source::deserialize(&result["source"]).map_err(crate::Error::WorktreeListDecode)?;
    let mut entries = Vec::with_capacity(rows.len());
    let mut paths = std::collections::HashSet::new();
    for row in rows {
        // Bound strings before deserializing/copying untrusted endpoint data.
        for field in ["path", "label", "branch", "open_workspace_id"] {
            if row[field]
                .as_str()
                .is_some_and(|text| text.len() > 8192 || text.contains('\0'))
            {
                return Err(crate::Error::WorktreeList);
            }
        }
        let entry = Entry::deserialize(row).map_err(crate::Error::WorktreeListDecode)?;
        if entry.path.is_empty() || !paths.insert(entry.path.clone()) {
            return Err(crate::Error::WorktreeList);
        }
        if !entry.is_bare && !entry.is_prunable {
            entries.push(entry);
        }
    }
    Ok((source, entries))
}

impl MenuState {
    pub(super) fn apply_worktree_list_response(
        &mut self,
        id: &str,
        result: crate::state::DialogResponse,
    ) {
        let Some(picker) = &mut self.worktree_open else {
            return;
        };
        if picker.pending.as_deref() != Some(id) {
            return;
        }
        picker.pending = None;
        self.error = match result {
            Err(error) => Some(error.to_string()),
            Ok(response) => {
                if let Some(refusal) = refusal(&response) {
                    Some(refusal)
                } else {
                    match listing(&response) {
                        Ok((source, entries)) => {
                            picker.source = Some(source);
                            picker.entries = entries;
                            picker.filter(&picker.query.clone());
                            None
                        }
                        Err(error) => Some(error.to_string()),
                    }
                }
            }
        };
    }
}

impl SheprdWindow {
    pub(super) fn worktree_open_current(&self) -> bool {
        if !self.menu_target_current() || !self.live.status.is_connected() {
            return false;
        }
        let Some((target, snapshot)) = self.menu.target.as_ref().zip(self.live.snapshot.as_ref())
        else {
            return false;
        };
        if snapshot.boot_id != target.boot_id {
            return false;
        }
        if target.validate_repository(snapshot).is_ok() {
            return true;
        }
        // Opening can establish the branch-only source's parent membership before
        // its response reaches the UI. Allow only the listed identity, while pending.
        if self.menu.page != Some(Page::Dialog(WorkspaceAction::OpenWorktree))
            || self.menu.opening.is_none()
            || target.worktree.is_some()
            || target.branch.is_none()
        {
            return false;
        }
        let Some(source) = self
            .menu
            .worktree_open
            .as_ref()
            .filter(|picker| picker.pending.is_none())
            .and_then(|picker| picker.source.as_ref())
        else {
            return false;
        };
        source.source_workspace_id.as_deref() == Some(target.id.as_str())
            && snapshot
                .workspaces
                .iter()
                .find(|workspace| workspace.workspace_id == target.id)
                .is_some_and(|workspace| {
                    workspace.branch == target.branch
                        && workspace.worktree.as_ref().is_some_and(|tree| {
                            !tree.is_linked_worktree
                                && tree.key == source.repo_key
                                && tree.label == source.repo_name
                        })
                })
    }

    pub(super) fn open_existing_worktrees(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let search = cx.new(SearchInput::new);
        search.update(cx, |input, cx| {
            input.set_placeholder_with(|| fl!("worktree-open-search"), cx);
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            window.focus(&input.focus, cx);
        });
        let subscription = cx.subscribe(
            &search,
            |this, search, _: &crate::search_input::Changed, cx| {
                if let Some(picker) = &mut this.menu.worktree_open {
                    picker.filter(search.read(cx).text());
                }
                cx.notify();
            },
        );
        let result = (|| {
            if !self.worktree_open_current() {
                return Err(crate::Error::StaleWorkspace);
            }
            let target = self
                .menu
                .target
                .as_ref()
                .ok_or(crate::Error::StaleWorkspace)?;
            self.endpoints[self.selected_endpoint]
                .connection
                .request_dialog(&target.boot_id, Method::WorktreeList, target.list_params())
        })();
        let mut picker = Picker::new(search);
        picker._subscription = Some(subscription);
        picker.pending = result.as_ref().ok().cloned();
        self.menu.worktree_open = Some(picker);
        self.menu.error = result.err().map(|error| error.to_string());
        cx.notify();
    }
}

#[cfg(test)]
mod tests;
