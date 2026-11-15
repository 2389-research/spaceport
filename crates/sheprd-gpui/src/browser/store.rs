// ABOUTME: Stores saved native tabs with exact runtime and workspace identities.
// ABOUTME: Preserves Core references while feature gates control their visibility.
//! Client-owned native tabs and the workspace each one belongs to.
//! Windows share tab references; the selected runtime owns their terminals.
use super::Location;
use crate::i18n::fl;
use crate::{reorder::Beside, state_file};
use gpui::{App, Global};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

mod saved;

const MAX_TABS: usize = 256;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_TITLE_CHARS: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct TabId(u64);

#[cfg(test)]
impl TabId {
    pub(crate) fn test(id: u64) -> Self {
        Self(id)
    }
}

/// An ID a control request names, as `tab list` printed it. Only ever
/// compared with the store's own IDs.
impl From<u64> for TabId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<TabId> for u64 {
    fn from(id: TabId) -> Self {
        id.0
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The daemon a workspace ID belongs to. IDs are only unique within one
/// daemon, so a local session and a saved host can both have a `w_1`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Scope {
    Legacy(String),
    Core(CoreScope),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScopeKind {
    Core,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CoreScope {
    kind: ScopeKind,
    hub_core_id: String,
    machine_id: Option<String>,
    document_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CoreGroupKey {
    scope: CoreScope,
    group_id: Uuid,
}

impl CoreGroupKey {
    pub(crate) fn new(
        hub_core_id: &str,
        machine_id: Option<&str>,
        document_name: &str,
        group_id: Uuid,
    ) -> Option<Self> {
        let key = Self {
            scope: CoreScope {
                kind: ScopeKind::Core,
                hub_core_id: hub_core_id.to_owned(),
                machine_id: machine_id.map(str::to_owned),
                document_name: document_name.to_owned(),
            },
            group_id,
        };
        key.valid().then_some(key)
    }

    pub(crate) fn group_id(&self) -> Uuid {
        self.group_id
    }

    pub(crate) fn hub_core_id(&self) -> &str {
        &self.scope.hub_core_id
    }

    pub(crate) fn machine_id(&self) -> Option<&str> {
        self.scope.machine_id.as_deref()
    }

    pub(crate) fn document_name(&self) -> &str {
        &self.scope.document_name
    }

    pub(crate) fn parts(&self) -> (Scope, String) {
        (Scope::Core(self.scope.clone()), self.group_id.to_string())
    }

    fn valid(&self) -> bool {
        let id = |value: &str| {
            !value.trim().is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
        };
        id(&self.scope.hub_core_id)
            && self.scope.machine_id.as_deref().is_none_or(id)
            && !self.scope.document_name.trim().is_empty()
            && self.scope.document_name.len() <= 128
            && !self.scope.document_name.chars().any(char::is_control)
            && !self.group_id.is_nil()
    }
}

impl Scope {
    /// A local daemon, named by the client socket the window connects to.
    pub(crate) fn local(client_socket: &std::path::Path) -> Self {
        Self::Legacy(format!("local:{}", client_socket.display()))
    }

    /// A saved SSH host, named by its catalog ID.
    pub(crate) fn endpoint(id: &str) -> Self {
        Self::Legacy(id.to_owned())
    }

    /// The scope as `tab list` reports a tab's host.
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Legacy(id) => id,
            Self::Core(core) => core.machine_id.as_deref().unwrap_or(&core.hub_core_id),
        }
    }

    pub(crate) fn core_group(&self, workspace_id: &str) -> Option<CoreGroupKey> {
        let Self::Core(scope) = self else {
            return None;
        };
        let key = CoreGroupKey {
            scope: scope.clone(),
            group_id: Uuid::parse_str(workspace_id).ok()?,
        };
        key.valid().then_some(key)
    }

    pub(crate) fn valid_workspace(&self, workspace_id: &str) -> bool {
        match self {
            Self::Legacy(_) => !workspace_id.is_empty() && workspace_id.len() <= 256,
            Self::Core(_) => self
                .core_group(workspace_id)
                .is_some_and(|key| key.group_id.to_string() == workspace_id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Tab {
    pub id: TabId,
    pub scope: Scope,
    pub workspace_id: String,
    /// `None` while a Core tab waits for its terminal reference.
    pub location: Option<Location>,
    /// The native content title, updated from terminal reports.
    pub title: String,
    /// The Herdr pane that opened the tab, when it has one.
    pub origin: Option<String>,
}

impl Tab {
    pub(crate) fn requires_core(&self) -> bool {
        matches!(self.scope, Scope::Core(_))
            || matches!(self.location, Some(Location::CoreTerminal { .. }))
    }

    /// The title the window shows: the one the app gives a tab it draws or
    /// one still waiting for its content. A terminal tab shows its program's
    /// title once the program sets one.
    pub(crate) fn label(&self) -> String {
        match &self.location {
            None => fl!("browser-tab-new"),
            Some(location @ Location::CoreTerminal { .. })
                if !self.title.is_empty() && self.title != location.default_title() =>
            {
                self.title.clone()
            }
            Some(location) => location.drawn_title(),
        }
    }

    pub(crate) fn core_group(&self) -> Option<CoreGroupKey> {
        self.scope.core_group(&self.workspace_id)
    }
}

/// The tabs as they are written, and as [`saved::parse`] restores them.
#[derive(Default, Serialize)]
struct Saved {
    tabs: Vec<Tab>,
    /// Preserve IDs of tabs removed by a later app version across saves.
    next: u64,
}

#[derive(Default)]
pub(crate) struct Store {
    tabs: Vec<Tab>,
    next: u64,
    pub(super) closed: super::closed::ClosedTabs,
    writer: Option<state_file::Writer<Saved>>,
    quitting: bool,
}

impl Global for Store {}

/// Page titles are untrusted: one line, bounded, and free of control and
/// bidirectional formatting characters that could disguise a tab.
fn clean_title(title: &str) -> String {
    crate::notifications::safe_text(title, MAX_TITLE_CHARS * 4)
        .chars()
        .take(MAX_TITLE_CHARS)
        .collect::<String>()
        .trim()
        .to_owned()
}

impl Store {
    #[cfg(test)]
    pub(crate) fn restored_for_test(bytes: &[u8]) -> crate::Result<Self> {
        Ok(Self::with_saved(saved::parse(bytes)?, None))
    }

    fn path() -> Option<PathBuf> {
        crate::preferences::state_dir().map(|dir| dir.join("browser-tabs.json"))
    }

    /// Called before starting GPUI, like the window state. A missing or
    /// damaged file starts with no tabs.
    pub(crate) fn load() -> Self {
        let path = Self::path();
        let tabs = path
            .as_deref()
            .map(|path| {
                state_file::read(path, MAX_FILE_BYTES)
                    .and_then(|bytes| bytes.as_deref().map_or(Ok(Saved::default()), saved::parse))
            })
            .transpose()
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "Cannot restore browser tabs");
                None
            })
            .unwrap_or_default();
        let writer = path.and_then(
            |path| match state_file::Writer::start("browser-tabs", path) {
                Ok(writer) => Some(writer),
                Err(error) => {
                    tracing::warn!(%error, "Cannot start browser-tab worker");
                    None
                }
            },
        );
        Self::with_saved(tabs, writer)
    }

    fn with_saved(saved: Saved, writer: Option<state_file::Writer<Saved>>) -> Self {
        Self {
            tabs: saved.tabs,
            next: saved.next,
            closed: Default::default(),
            writer,
            quitting: false,
        }
    }

    pub(crate) fn install(self, cx: &mut App) {
        cx.set_global(self);
        cx.on_app_quit(|cx| {
            let store = cx.global_mut::<Self>();
            store.quitting = true;
            let writer = store.writer.take();
            cx.background_executor().spawn(async move {
                if let Some(writer) = writer {
                    writer.finish();
                }
            })
        })
        .detach();
    }

    /// Runs `f` against the app's store, creating an unsaved one for
    /// fixtures that never installed it.
    pub(crate) fn update<R>(cx: &mut App, f: impl FnOnce(&mut Self) -> R) -> R {
        if !cx.has_global::<Self>() {
            cx.set_global(Self::default());
        }
        f(cx.global_mut::<Self>())
    }

    fn save(&self) {
        if self.quitting {
            return;
        }
        if let Some(writer) = &self.writer {
            writer.save(self.saved());
        }
    }

    fn saved(&self) -> Saved {
        Saved {
            tabs: self.tabs.clone(),
            next: self.next,
        }
    }

    /// Every tab of every workspace, in the order they were opened.
    pub(crate) fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    pub(crate) fn get(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    /// The tab strip tabs of one workspace, in the order they were opened.
    pub(crate) fn in_workspace<'a>(
        &'a self,
        scope: &'a Scope,
        workspace_id: &'a str,
    ) -> impl Iterator<Item = &'a Tab> + 'a {
        self.tabs
            .iter()
            .filter(move |tab| &tab.scope == scope && tab.workspace_id == workspace_id)
    }

    pub(crate) fn in_core_group<'a>(
        &'a self,
        key: &'a CoreGroupKey,
    ) -> impl Iterator<Item = &'a Tab> + 'a {
        self.tabs
            .iter()
            .filter(move |tab| tab.core_group().as_ref() == Some(key))
    }

    /// Opens a tab for the strips, or returns `None` when the app already
    /// holds the most it keeps.
    pub(crate) fn open(
        &mut self,
        scope: Scope,
        workspace_id: &str,
        location: Option<Location>,
        origin: Option<String>,
    ) -> Option<TabId> {
        let title = location
            .as_ref()
            // i18n: data
            .map_or_else(|| "New Tab".to_owned(), Location::default_title);
        self.restore_tab(Tab {
            id: TabId(0),
            scope,
            workspace_id: workspace_id.to_owned(),
            location,
            title,
            origin,
        })
    }

    pub(crate) fn open_core(
        &mut self,
        key: &CoreGroupKey,
        location: Option<Location>,
    ) -> Option<TabId> {
        let (scope, workspace_id) = key.parts();
        self.open(scope, &workspace_id, location, None)
    }

    pub(super) fn restore_tab(&mut self, mut tab: Tab) -> Option<TabId> {
        if self.tabs.len() >= MAX_TABS {
            return None;
        }
        let following = self.next.checked_add(1)?;
        let id = TabId(self.next);
        self.next = following;
        tab.id = id;
        self.tabs.push(tab);
        self.save();
        Some(id)
    }

    /// Moves a tab beside another; whether it moved. Workspaces list their
    /// tabs in store order, so landing beside one of the same workspace
    /// reorders that workspace's strip alone.
    pub(crate) fn move_tab(&mut self, id: TabId, beside: Beside<TabId>) -> bool {
        let (Beside::Before(anchor) | Beside::After(anchor)) = beside;
        if anchor == id {
            return false;
        }
        let Some(from) = self.tabs.iter().position(|tab| tab.id == id) else {
            return false;
        };
        let tab = self.tabs.remove(from);
        let Some(at) = self.tabs.iter().position(|tab| tab.id == anchor) else {
            self.tabs.insert(from, tab);
            return false;
        };
        let to = match beside {
            Beside::Before(_) => at,
            Beside::After(_) => at + 1,
        };
        self.tabs.insert(to, tab);
        if to == from {
            return false;
        }
        self.save();
        true
    }

    pub(crate) fn close(&mut self, id: TabId) -> Option<Tab> {
        let tab = self.detach(id)?;
        self.closed.push_page(tab.clone());
        Some(tab)
    }

    /// Removes a tab without recording it as closed, as when its view moves
    /// into a detached window of its own.
    pub(crate) fn detach(&mut self, id: TabId) -> Option<Tab> {
        let index = self.tabs.iter().position(|tab| tab.id == id)?;
        let tab = self.tabs.remove(index);
        self.save();
        Some(tab)
    }

    /// Updates a native content reference or title. Returns whether it changed.
    pub(crate) fn visited(
        &mut self,
        id: TabId,
        location: Option<Location>,
        title: Option<&str>,
    ) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) else {
            return false;
        };
        let mut changed = false;
        if let Some(location) = location
            && tab.location.as_ref() != Some(&location)
        {
            tab.location = Some(location);
            changed = true;
        }
        if let Some(title) = title.map(clean_title).filter(|title| !title.is_empty())
            && tab.title != title
        {
            tab.title = title;
            changed = true;
        }
        if changed {
            self.save();
        }
        changed
    }

    /// Changes a Core terminal tab's pane tree in place, and saves when the
    /// tree changed. Every window draws this one tree. `None` when `id` is
    /// not a Core terminal tab.
    pub(crate) fn edit_panes<R>(
        &mut self,
        id: TabId,
        edit: impl FnOnce(&mut crate::core_terminals::panes::Panes) -> R,
    ) -> Option<R> {
        let tab = self.tabs.iter_mut().find(|tab| tab.id == id)?;
        let Some(Location::CoreTerminal { panes, .. }) = &mut tab.location else {
            return None;
        };
        let before = panes.clone();
        let result = edit(panes);
        if *panes != before {
            self.save();
        }
        Some(result)
    }

    /// Drops the tabs of workspaces the daemon closed.
    pub(crate) fn forget_workspaces(&mut self, scope: &Scope, closed: &[String]) -> bool {
        self.closed.forget_workspaces(scope, closed);
        let before = self.tabs.len();
        self.tabs
            .retain(|tab| &tab.scope != scope || !closed.contains(&tab.workspace_id));
        let changed = self.tabs.len() != before;
        if changed {
            self.save();
        }
        changed
    }

    /// Whether any tab belongs to one of `closed`, without claiming the
    /// store for writing, which would redraw every window.
    pub(crate) fn has_workspaces(&self, scope: &Scope, closed: &[String]) -> bool {
        self.closed.has_workspaces(scope, closed)
            || self
                .tabs
                .iter()
                .any(|tab| &tab.scope == scope && closed.contains(&tab.workspace_id))
    }
}

#[cfg(test)]
mod tests;
