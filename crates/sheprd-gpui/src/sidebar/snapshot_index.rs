// ABOUTME: Indexes snapshot row positions and stable agent orders without copying records.
// ABOUTME: Reuses exact snapshot identities through a weak, transport-safe presentation cache.
use crate::{preferences::AgentSort, state::LiveState};
use sheprd_client::protocol::{
    ClientShellAgent, ClientShellPane, ClientShellSnapshot, ClientShellTab, ClientShellWorkspace,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Weak},
};

#[derive(Clone, Default)]
pub(crate) struct SnapshotIndexCache(RefCell<Option<Cached>>);

#[derive(Clone)]
struct Cached {
    source: Weak<ClientShellSnapshot>,
    index: Arc<SnapshotIndex>,
}

#[derive(Default)]
struct Order {
    all: Vec<usize>,
    workspaces: HashMap<String, Vec<usize>>,
}

struct SnapshotIndex {
    agents: HashMap<String, usize>,
    workspaces: HashMap<String, usize>,
    tabs: HashMap<String, usize>,
    panes: HashMap<String, usize>,
    workspace_tabs: HashMap<String, Vec<usize>>,
    tab_panes: HashMap<String, HashMap<String, Vec<usize>>>,
    first_tab: HashMap<String, usize>,
    grouped: Order,
    priority: Order,
}

pub(crate) struct IndexedSnapshot<'a> {
    snapshot: &'a ClientShellSnapshot,
    index: Arc<SnapshotIndex>,
}

impl SnapshotIndexCache {
    fn for_snapshot(&self, snapshot: &Arc<ClientShellSnapshot>) -> Arc<SnapshotIndex> {
        if let Some(cached) = self.0.borrow().as_ref()
            && cached.source.as_ptr() == Arc::as_ptr(snapshot)
        {
            return cached.index.clone();
        }
        let index = Arc::new(SnapshotIndex::new(snapshot));
        *self.0.borrow_mut() = Some(Cached {
            source: Arc::downgrade(snapshot),
            index: index.clone(),
        });
        index
    }

    pub(crate) fn indexed<'a>(
        &self,
        snapshot: &'a Arc<ClientShellSnapshot>,
    ) -> IndexedSnapshot<'a> {
        IndexedSnapshot::with_index(snapshot, self.for_snapshot(snapshot))
    }
}

impl Order {
    fn new(snapshot: &ClientShellSnapshot, all: Vec<usize>) -> Self {
        let mut workspaces: HashMap<String, Vec<usize>> = HashMap::new();
        for &position in &all {
            #[cfg(test)]
            super::agents::visits::record();
            let agent = &snapshot.agents[position];
            workspaces
                .entry(agent.workspace_id.clone())
                .or_default()
                .push(position);
        }
        Self { all, workspaces }
    }
}

impl SnapshotIndex {
    fn new(snapshot: &ClientShellSnapshot) -> Self {
        let mut agents = HashMap::new();
        let mut workspaces = HashMap::new();
        let mut tabs = HashMap::new();
        let mut panes = HashMap::new();
        let mut workspace_tabs: HashMap<String, Vec<usize>> = HashMap::new();
        let mut tab_panes: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();
        let mut first_tab: HashMap<String, usize> = HashMap::new();
        for (position, agent) in snapshot.agents.iter().enumerate() {
            #[cfg(test)]
            super::agents::visits::record();
            agents.entry(agent.pane_id.clone()).or_insert(position);
        }
        for (position, workspace) in snapshot.workspaces.iter().enumerate() {
            #[cfg(test)]
            super::agents::visits::record();
            workspaces
                .entry(workspace.workspace_id.clone())
                .or_insert(position);
        }
        for (position, tab) in snapshot.tabs.iter().enumerate() {
            #[cfg(test)]
            super::agents::visits::record();
            tabs.entry(tab.tab_id.clone()).or_insert(position);
            workspace_tabs
                .entry(tab.workspace_id.clone())
                .or_default()
                .push(position);
            let first = first_tab
                .entry(tab.workspace_id.clone())
                .or_insert(position);
            if tab.number < snapshot.tabs[*first].number {
                *first = position;
            }
        }
        for (position, pane) in snapshot.panes.iter().enumerate() {
            #[cfg(test)]
            super::agents::visits::record();
            panes.entry(pane.pane_id.clone()).or_insert(position);
            tab_panes
                .entry(pane.workspace_id.clone())
                .or_default()
                .entry(pane.tab_id.clone())
                .or_default()
                .push(position);
        }
        let grouped: Vec<usize> = if snapshot.agent_view_label.is_some() {
            snapshot
                .agent_order
                .iter()
                .filter_map(|pane| {
                    #[cfg(test)]
                    super::agents::visits::record();
                    agents.get(pane).copied()
                })
                .collect()
        } else {
            (0..snapshot.agents.len()).collect()
        };
        let mut priority = grouped.clone();
        if snapshot.agent_view_label.is_none() {
            priority.sort_by_key(|&position| {
                let agent = &snapshot.agents[position];
                (
                    std::cmp::Reverse(super::agents::status_priority(agent.agent_status)),
                    std::cmp::Reverse(agent.state_change_seq),
                )
            });
        }
        Self {
            agents,
            workspaces,
            tabs,
            panes,
            workspace_tabs,
            tab_panes,
            first_tab,
            grouped: Order::new(snapshot, grouped),
            priority: Order::new(snapshot, priority),
        }
    }
}

impl<'a> IndexedSnapshot<'a> {
    pub(crate) fn snapshot(&self) -> &'a ClientShellSnapshot {
        self.snapshot
    }

    fn with_index(snapshot: &'a ClientShellSnapshot, index: Arc<SnapshotIndex>) -> Self {
        Self { snapshot, index }
    }

    #[cfg(test)]
    pub(crate) fn new(snapshot: &'a ClientShellSnapshot) -> Self {
        Self::with_index(snapshot, Arc::new(SnapshotIndex::new(snapshot)))
    }

    fn order(&self, sort: AgentSort) -> &Order {
        match sort {
            AgentSort::Grouped => &self.index.grouped,
            AgentSort::Priority => &self.index.priority,
        }
    }

    pub(crate) fn agents(&self, sort: AgentSort) -> Vec<&'a ClientShellAgent> {
        self.order(sort)
            .all
            .iter()
            .map(|&position| &self.snapshot.agents[position])
            .collect()
    }

    pub(crate) fn workspace_agents(
        &self,
        workspace: &str,
        sort: AgentSort,
    ) -> Vec<&'a ClientShellAgent> {
        self.order(sort)
            .workspaces
            .get(workspace)
            .into_iter()
            .flatten()
            .map(|&position| &self.snapshot.agents[position])
            .collect()
    }

    pub(crate) fn agent(&self, pane: &str) -> Option<&'a ClientShellAgent> {
        self.snapshot.agents.get(*self.index.agents.get(pane)?)
    }
    pub(crate) fn workspace(&self, id: &str) -> Option<&'a ClientShellWorkspace> {
        self.snapshot
            .workspaces
            .get(*self.index.workspaces.get(id)?)
    }
    pub(crate) fn tab(&self, id: &str) -> Option<&'a ClientShellTab> {
        self.snapshot.tabs.get(*self.index.tabs.get(id)?)
    }
    pub(crate) fn pane(&self, id: &str) -> Option<&'a ClientShellPane> {
        self.snapshot.panes.get(*self.index.panes.get(id)?)
    }
    pub(crate) fn tab_count(&self, workspace: &str) -> usize {
        self.index.workspace_tabs.get(workspace).map_or(0, Vec::len)
    }
    pub(crate) fn tabs(&self, workspace: &str) -> Vec<&'a ClientShellTab> {
        self.index
            .workspace_tabs
            .get(workspace)
            .into_iter()
            .flatten()
            .map(|&position| &self.snapshot.tabs[position])
            .collect()
    }
    pub(crate) fn panes(
        &self,
        workspace: &str,
        tab: &str,
    ) -> impl Iterator<Item = &'a ClientShellPane> + '_ {
        self.index
            .tab_panes
            .get(workspace)
            .and_then(|tabs| tabs.get(tab))
            .into_iter()
            .flatten()
            .map(|&position| &self.snapshot.panes[position])
    }
    pub(crate) fn launch_root(&self, workspace: &str) -> Option<&'a str> {
        let tab = self
            .snapshot
            .tabs
            .get(*self.index.first_tab.get(workspace)?)?;
        self.panes(workspace, &tab.tab_id).next()?.cwd.as_deref()
    }
}

impl LiveState {
    pub(crate) fn indexed_snapshot(&self) -> Option<IndexedSnapshot<'_>> {
        let snapshot = self.snapshot.as_ref()?;
        Some(self.sidebar_index.indexed(snapshot))
    }
}

#[cfg(test)]
mod tests;
