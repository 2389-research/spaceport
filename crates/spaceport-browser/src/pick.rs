//! What each group of the focused workspace lists and shows: the rules in
//! [`super::groups`], applied to the workspace, the endpoint and the store the
//! window hands over, and the Herdr tabs the daemon reports.
use super::{Browser, GroupId, Layout, Pick, Scope, Slot, hosts::BrowserHosts};
use std::collections::hash_map::Entry;

impl Browser {
    /// The daemon's focused tab, when it belongs to the focused workspace.
    pub fn focused_herdr_tab<'a>(&self, hosts: &BrowserHosts<'a>) -> Option<&'a str> {
        let snapshot = hosts.snapshot?;
        let tab = snapshot.focused_tab_id.as_deref()?;
        snapshot
            .tabs
            .iter()
            .any(|candidate| {
                candidate.tab_id == tab
                    && Some(&candidate.workspace_id) == snapshot.focused_workspace_id.as_ref()
            })
            .then_some(tab)
    }

    pub fn layout<'a>(&'a self, hosts: &BrowserHosts<'_>) -> Option<&'a Layout> {
        self.layouts.get(&self.layout_key(hosts)?)
    }

    /// `key`'s groups, made on first use from the layout saved for it, if
    /// any, or else as one group following the terminal.
    pub fn layout_for(&mut self, key: (Scope, String)) -> &mut Layout {
        match self.layouts.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let layout = match self.saved.remove(entry.key()) {
                    Some(saved) => {
                        self.restored = Some(entry.key().clone());
                        Layout::restore(&saved, &mut self.group_ids)
                    }
                    None => Layout::new(self.group_ids.next()),
                };
                entry.insert(layout)
            }
        }
    }

    /// The focused workspace's groups, creating its first one. Render calls
    /// this before drawing so every group it draws has an ID.
    pub fn ensure_layout<'a>(&'a mut self, hosts: &BrowserHosts<'_>) -> Option<&'a mut Layout> {
        let key = self.layout_key(hosts)?;
        Some(self.layout_for(key))
    }

    /// The focused workspace's groups, left to right. Without a workspace
    /// the window still draws one group, for the terminal area.
    pub fn group_slots(&self, hosts: &BrowserHosts<'_>) -> Vec<Slot> {
        match self.layout(hosts) {
            Some(layout) => layout.slots().collect(),
            None => vec![Slot {
                id: self.fallback_group,
                index: 0,
            }],
        }
    }

    pub fn is_split(&self, hosts: &BrowserHosts<'_>) -> bool {
        self.layout(hosts).is_some_and(|layout| layout.len() > 1)
    }

    pub fn active_group(&self, hosts: &BrowserHosts<'_>) -> Option<GroupId> {
        self.layout(hosts).map(Layout::active)
    }

    /// The share of the row `group` takes as drawn now: its settled share,
    /// bent while a group beside it opens or folds.
    pub fn group_share(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> f32 {
        let Some(layout) = self.layout(hosts) else {
            return 1.;
        };
        self.group_motion.share(
            group,
            layout.share(group),
            |other| layout.share(other),
            hosts.now,
        )
    }

    /// How far `group` has opened from its split, or `None` once open.
    pub fn group_opened(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<f32> {
        self.group_motion.opened(group, hosts.now)
    }

    /// The closed groups still folding away.
    pub fn folding_groups(&self, hosts: &BrowserHosts<'_>) -> Vec<super::Fold> {
        self.group_motion.folding(hosts.now)
    }

    /// Whether a group is still opening or folding, so the window draws
    /// another frame.
    pub fn groups_moving(&mut self, hosts: &BrowserHosts<'_>) -> bool {
        self.group_motion.animating(hosts.now)
    }

    /// The tab `group` picked, the focused one when it follows the terminal.
    pub fn group_pick(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<Pick> {
        let focused = hosts
            .core_group
            .is_none()
            .then(|| self.focused_herdr_tab(hosts))
            .flatten();
        self.layout(hosts)?.pick(group, focused)
    }

    /// Whether `group`'s strip lists `pick`; see [`Layout::lists`]. A tab
    /// this window hides is listed only while the group shows it.
    pub fn group_lists(&self, group: GroupId, pick: &Pick, hosts: &BrowserHosts<'_>) -> bool {
        if self.hides_tab(pick, hosts) && self.shown_pick(group, hosts).as_ref() != Some(pick) {
            return false;
        }
        let Some(layout) = self.layout(hosts) else {
            return true;
        };
        layout.lists(group, pick)
    }

    /// What `group` shows, reading a window without groups yet as one
    /// following the terminal.
    pub(crate) fn shown_pick(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<Pick> {
        let focused = hosts
            .core_group
            .is_none()
            .then(|| self.focused_herdr_tab(hosts))
            .flatten();
        match self.layout(hosts) {
            Some(layout) => layout.pick(group, focused),
            None => focused.map(|tab| Pick::Herdr(tab.to_owned())),
        }
    }

    /// Whether this window keeps `pick` out of the focused workspace's
    /// strips; see [`super::hidden`].
    pub(crate) fn hides_tab(&self, pick: &Pick, hosts: &BrowserHosts<'_>) -> bool {
        !self.hidden.is_empty()
            && self
                .layout_key(hosts)
                .is_some_and(|key| self.hidden.hides(&key, pick))
    }

    /// The focused workspace's tabs this window lists, in strip order:
    /// Herdr tabs first, then browser tabs. A hidden tab counts while a
    /// group shows it.
    pub fn workspace_tabs(&self, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
        let slots = self.group_slots(hosts);
        self.open_workspace_tabs(hosts)
            .into_iter()
            .filter(|pick| {
                !self.hides_tab(pick, hosts)
                    || slots
                        .iter()
                        .any(|slot| self.shown_pick(slot.id, hosts).as_ref() == Some(pick))
            })
            .collect()
    }

    /// Every tab of the focused workspace, hidden ones included.
    pub fn open_workspace_tabs(&self, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
        if hosts.workspace.is_none() {
            return Vec::new();
        }
        let herdr = hosts
            .core_group
            .is_none()
            .then_some(hosts.snapshot)
            .flatten()
            .map(|snapshot| {
                snapshot
                    .tabs
                    .iter()
                    .filter(|tab| Some(&tab.workspace_id) == snapshot.focused_workspace_id.as_ref())
                    .map(|tab| Pick::Herdr(tab.tab_id.clone()))
                    .collect::<Vec<_>>()
            });
        herdr
            .unwrap_or_default()
            .into_iter()
            .chain(self.browser_tab_ids(hosts).into_iter().map(Pick::Page))
            .collect()
    }

    /// The tabs `group`'s strip lists, in strip order.
    pub fn group_tabs(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
        self.workspace_tabs(hosts)
            .into_iter()
            .filter(|pick| self.group_lists(group, pick, hosts))
            .collect()
    }

    /// The next Herdr tab the daemon focuses goes to `group`: its "+" asked
    /// for it.
    pub fn expect_new_tab_in(&mut self, group: GroupId, hosts: &BrowserHosts<'_>) {
        self.new_tab_group = self.browser_key(hosts).map(|key| (key, group));
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn expected_new_tab_group(&self) -> Option<GroupId> {
        self.new_tab_group.as_ref().map(|(_, group)| *group)
    }

    /// Moves the divider after the `divider`th group; see [`Layout::drag`].
    pub fn drag_divider(
        &mut self,
        divider: usize,
        offset: f32,
        width: f32,
        hosts: &BrowserHosts<'_>,
    ) -> bool {
        self.ensure_layout(hosts)
            .is_some_and(|layout| layout.drag(divider, offset, width))
    }
}
