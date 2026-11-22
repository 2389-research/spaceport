//! One window's browser: how each of its workspaces is split into groups, and
//! which tab each group shows. The tabs themselves are the app's; this reads
//! them through [`BrowserHosts`] and keeps no handle of its own.
use super::{
    ClosedHerdrTab, CoreGroupKey, GroupId, GroupIds, GroupMotion, Hidden, Layout, Pick,
    SavedLayout, Scope, Tab, TabAppear, TabBodies, TabId, TabScroll, close_run::CloseRun,
    hosts::BrowserHosts,
};
use gpui::{Bounds, Pixels};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// One window's browser state. How each workspace's view is split, and what
/// each group shows, is the window's own choice, like its focused workspace;
/// the tabs themselves are the app's.
pub struct Browser {
    /// Per workspace, its groups. A workspace missing here has one group
    /// following its terminal.
    pub layouts: HashMap<(Scope, String), Layout>,
    /// Layouts saved before this window opened, restored the first time it
    /// shows each workspace.
    pub saved: HashMap<(Scope, String), SavedLayout>,
    /// The workspace whose layout was just restored, whose group in use is
    /// taken back to its tab once, rather than following wherever the
    /// daemon's focus was left.
    pub restored: Option<(Scope, String)>,
    /// The IDs handed to the groups this window makes, in order.
    pub group_ids: GroupIds,
    /// The group drawn before the window has a workspace to split.
    pub fallback_group: GroupId,
    /// Groups whose shown tab closed where their strips were not at hand:
    /// in another window, or in a workspace not on screen. Each is given
    /// another of its tabs, or closed, when its workspace next settles.
    pub emptied: Vec<GroupId>,
    /// Tabs this window keeps out of its strips, as tearing one off does.
    pub hidden: Hidden,
    /// Tabs being closed one after another, as Close Others closes them.
    pub closing: Option<CloseRun>,
    /// Where each group drew last frame, so a carried tab knows the group
    /// it is over.
    pub group_bounds: Rc<RefCell<HashMap<GroupId, Bounds<Pixels>>>>,
    /// The group whose "+" asked for a new Herdr tab, which it picks once
    /// the daemon focuses the tab.
    pub new_tab_group: Option<((Scope, String), GroupId)>,
    /// Tabs growing into the strip as they open.
    pub appear: TabAppear,
    /// Each strip's sideways scroll.
    pub tab_scroll: TabScroll,
    /// Each strip's connected tab body, for layouts that draw one.
    pub tab_body: TabBodies,
    /// Groups opening from a split and folding away as they close.
    pub group_motion: GroupMotion,
    /// The workspaces of the last snapshot and the boot they came from: one
    /// missing from the next snapshot of the same boot was closed.
    pub workspaces: Option<(Scope, String, HashSet<String>)>,
    /// The Herdr tabs of the last snapshot, with its boot and revision: one
    /// missing from the next snapshot of the same boot, in a workspace still
    /// open, was closed and can be reopened.
    pub herdr_tabs: Option<(Scope, String, u64, Vec<ClosedHerdrTab>)>,
}

impl Browser {
    /// A window's browser, starting with the layouts the app saved for each
    /// workspace; each is restored as the window first shows it.
    pub fn new(saved: HashMap<(Scope, String), SavedLayout>) -> Self {
        let mut group_ids = GroupIds::default();
        Self {
            layouts: HashMap::new(),
            saved,
            restored: None,
            fallback_group: group_ids.next(),
            group_ids,
            emptied: Vec::new(),
            hidden: Default::default(),
            closing: None,
            group_bounds: Default::default(),
            new_tab_group: None,
            appear: Default::default(),
            tab_scroll: Default::default(),
            tab_body: Default::default(),
            group_motion: Default::default(),
            workspaces: None,
            herdr_tabs: None,
        }
    }

    pub fn browser_key(&self, hosts: &BrowserHosts<'_>) -> Option<(Scope, String)> {
        if hosts.core_group.is_some() {
            return None;
        }
        let workspace = hosts.snapshot?.focused_workspace_id.clone()?;
        Some((hosts.scope.clone()?, workspace))
    }

    /// The group drawn in this window, which may belong to Core instead of
    /// the legacy endpoint that still supplies the transitional shell.
    pub(crate) fn layout_key(&self, hosts: &BrowserHosts<'_>) -> Option<(Scope, String)> {
        hosts
            .core_group
            .map(CoreGroupKey::parts)
            .or_else(|| self.browser_key(hosts))
    }

    /// The focused workspace's browser tabs, in the order they opened.
    pub fn browser_tabs<'a>(&self, hosts: &'a BrowserHosts<'a>) -> Vec<&'a Tab> {
        let (Some((scope, workspace)), Some(store)) = (hosts.workspace.as_ref(), hosts.tabs) else {
            return Vec::new();
        };
        let core_enabled = hosts.core_enabled;
        store
            .in_workspace(scope, workspace)
            .filter(move |tab| core_enabled || !tab.requires_core())
            .collect()
    }

    /// The focused workspace's browser tabs' IDs, in the order they opened.
    pub fn browser_tab_ids(&self, hosts: &BrowserHosts<'_>) -> Vec<TabId> {
        self.browser_tabs(hosts)
            .into_iter()
            .map(|tab| tab.id)
            .collect()
    }

    pub fn forget_browser_tabs(&mut self, mut gone: impl FnMut(TabId) -> bool) {
        for layout in self.layouts.values_mut() {
            let closed: Vec<Pick> = layout
                .tabs()
                .filter(|pick| matches!(pick, Pick::Page(id) if gone(*id)))
                .cloned()
                .collect();
            for pick in closed {
                self.emptied.extend(layout.forget(&pick));
            }
        }
    }
}
