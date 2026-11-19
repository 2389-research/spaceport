// ABOUTME: Routes native tab selection and split layouts within a window.
// ABOUTME: Keeps disabled Core references out of visible strips and selection.
//! The window's editor groups: which group shows which tab, using and
//! splitting groups, and following the daemon's focus. The rules for what a
//! group draws live in [`super::groups`]; this applies them to the focused
//! workspace and carries out what they imply, such as focusing the Herdr tab
//! a group asks for.
use super::{
    Browser, Location, Scope, TabId,
    groups::{GroupId, Layout, Pick, Shown, Slot},
    hosts::BrowserHosts,
    view::store,
};
use crate::{NavigationTarget, SheprdWindow, controls::Command};
use gpui::{prelude::*, *};
use std::collections::hash_map::Entry;

/// The tab a strip listing `tabs` goes to when `pick` leaves it: the nearest
/// one after it that `near` accepts, or else before it.
fn neighbour(tabs: &[Pick], pick: &Pick, near: impl Fn(&Pick) -> bool) -> Option<Pick> {
    let position = tabs.iter().position(|tab| tab == pick)?;
    tabs[position + 1..]
        .iter()
        .find(|tab| near(tab))
        .or_else(|| tabs[..position].iter().rev().find(|tab| near(tab)))
        .cloned()
}

/// The tab Next Tab, Previous Tab, or a tab by number lands on among `tabs`
/// from `current`. Next and Previous wrap around, and a number counts from
/// one, as Herdr's `switch_tab` does.
fn stepped<'a>(tabs: &'a [Pick], current: Option<&Pick>, command: Command) -> Option<&'a Pick> {
    let index = match command {
        Command::TabNumber(number) => usize::from(number).checked_sub(1)?,
        Command::NextTab | Command::PreviousTab => {
            let index = tabs.iter().position(|tab| Some(tab) == current)?;
            let step = if command == Command::NextTab {
                1
            } else {
                tabs.len() - 1
            };
            (index + step) % tabs.len()
        }
        _ => return None,
    };
    tabs.get(index)
}

impl Browser {
    /// The daemon's focused tab, when it belongs to the focused workspace.
    pub(crate) fn focused_herdr_tab<'a>(&self, hosts: &BrowserHosts<'a>) -> Option<&'a str> {
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

    fn layout<'a>(&'a self, hosts: &BrowserHosts<'_>) -> Option<&'a Layout> {
        self.layouts.get(&self.layout_key(hosts)?)
    }

    /// `key`'s groups, made on first use from the layout saved for it, if
    /// any, or else as one group following the terminal.
    fn layout_for(&mut self, key: (Scope, String)) -> &mut Layout {
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
    pub(crate) fn ensure_layout<'a>(
        &'a mut self,
        hosts: &BrowserHosts<'_>,
    ) -> Option<&'a mut Layout> {
        let key = self.layout_key(hosts)?;
        Some(self.layout_for(key))
    }

    /// The focused workspace's groups, left to right. Without a workspace
    /// the window still draws one group, for the terminal area.
    pub(crate) fn group_slots(&self, hosts: &BrowserHosts<'_>) -> Vec<Slot> {
        match self.layout(hosts) {
            Some(layout) => layout.slots().collect(),
            None => vec![Slot {
                id: self.fallback_group,
                index: 0,
            }],
        }
    }

    pub(crate) fn is_split(&self, hosts: &BrowserHosts<'_>) -> bool {
        self.layout(hosts).is_some_and(|layout| layout.len() > 1)
    }

    pub(crate) fn active_group(&self, hosts: &BrowserHosts<'_>) -> Option<GroupId> {
        self.layout(hosts).map(Layout::active)
    }

    /// The share of the row `group` takes as drawn now: its settled share,
    /// bent while a group beside it opens or folds.
    pub(crate) fn group_share(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> f32 {
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
    pub(crate) fn group_opened(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<f32> {
        self.group_motion.opened(group, hosts.now)
    }

    /// The closed groups still folding away.
    pub(crate) fn folding_groups(&self, hosts: &BrowserHosts<'_>) -> Vec<super::Fold> {
        self.group_motion.folding(hosts.now)
    }

    /// Whether a group is still opening or folding, so the window draws
    /// another frame.
    pub(crate) fn groups_moving(&mut self, hosts: &BrowserHosts<'_>) -> bool {
        self.group_motion.animating(hosts.now)
    }

    /// The tab `group` picked, the focused one when it follows the terminal.
    pub(crate) fn group_pick(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<Pick> {
        let focused = hosts
            .core_group
            .is_none()
            .then(|| self.focused_herdr_tab(hosts))
            .flatten();
        self.layout(hosts)?.pick(group, focused)
    }

    /// Whether `group`'s strip lists `pick`; see [`Layout::lists`]. A tab
    /// this window hides is listed only while the group shows it.
    pub(crate) fn group_lists(
        &self,
        group: GroupId,
        pick: &Pick,
        hosts: &BrowserHosts<'_>,
    ) -> bool {
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
    fn shown_pick(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Option<Pick> {
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
    fn hides_tab(&self, pick: &Pick, hosts: &BrowserHosts<'_>) -> bool {
        !self.hidden.is_empty()
            && self
                .layout_key(hosts)
                .is_some_and(|key| self.hidden.hides(&key, pick))
    }

    /// The focused workspace's tabs this window lists, in strip order:
    /// Herdr tabs first, then browser tabs. A hidden tab counts while a
    /// group shows it.
    pub(super) fn workspace_tabs(&self, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
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
    pub(crate) fn open_workspace_tabs(&self, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
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
    pub(crate) fn group_tabs(&self, group: GroupId, hosts: &BrowserHosts<'_>) -> Vec<Pick> {
        self.workspace_tabs(hosts)
            .into_iter()
            .filter(|pick| self.group_lists(group, pick, hosts))
            .collect()
    }

    /// The next Herdr tab the daemon focuses goes to `group`: its "+" asked
    /// for it.
    pub(crate) fn expect_new_tab_in(&mut self, group: GroupId, hosts: &BrowserHosts<'_>) {
        self.new_tab_group = self.browser_key(hosts).map(|key| (key, group));
    }

    #[cfg(test)]
    pub(crate) fn expected_new_tab_group(&self) -> Option<GroupId> {
        self.new_tab_group.as_ref().map(|(_, group)| *group)
    }

    /// The focused workspace's browser tabs and the labels their strips
    /// show, in strip order.
    pub(crate) fn browser_tab_labels(&self, hosts: &BrowserHosts<'_>) -> Vec<(Pick, SharedString)> {
        let (Some((scope, workspace)), Some(store)) = (hosts.workspace.as_ref(), hosts.tabs) else {
            return Vec::new();
        };
        store
            .in_workspace(scope, workspace)
            .filter(|tab| hosts.core_enabled || !tab.requires_core())
            .map(|tab| (Pick::Page(tab.id), super::view::tab_label(&tab.label())))
            .collect()
    }

    /// Moves the divider after the `divider`th group; see [`Layout::drag`].
    pub(crate) fn drag_divider(
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

impl SheprdWindow {
    /// Whether some group of the focused workspace shows the page `id`.
    pub(crate) fn group_shows_page(&self, id: TabId, cx: &App) -> bool {
        self.group_slots()
            .into_iter()
            .any(|slot| self.group_shown(slot.id, cx) == Shown::Page(id))
    }
    pub(crate) fn focused_herdr_tab(&self) -> Option<&str> {
        self.browser.focused_herdr_tab(&self.browser_hosts(None))
    }

    fn layout(&self) -> Option<&Layout> {
        self.browser.layout(&self.browser_hosts(None))
    }

    fn layout_for(&mut self, key: (Scope, String)) -> &mut Layout {
        let (browser, _) = self.browser_parts(None);
        browser.layout_for(key)
    }

    /// Saves each of this window's layouts that changed, for the next start.
    pub(crate) fn save_group_layouts(&mut self, cx: &mut Context<Self>) {
        for (key, layout) in &self.browser.layouts {
            super::Layouts::record(cx, key, layout.saved());
        }
    }

    pub(crate) fn ensure_layout(&mut self) -> Option<&mut Layout> {
        let (browser, hosts) = self.browser_parts(None);
        browser.ensure_layout(&hosts)
    }

    pub(crate) fn group_slots(&self) -> Vec<Slot> {
        self.browser.group_slots(&self.browser_hosts(None))
    }

    pub(crate) fn is_split(&self) -> bool {
        self.browser.is_split(&self.browser_hosts(None))
    }

    pub(crate) fn active_group(&self) -> Option<GroupId> {
        self.browser.active_group(&self.browser_hosts(None))
    }

    pub(crate) fn group_share(&self, group: GroupId) -> f32 {
        self.browser.group_share(group, &self.browser_hosts(None))
    }

    pub(crate) fn group_opened(&self, group: GroupId) -> Option<f32> {
        self.browser.group_opened(group, &self.browser_hosts(None))
    }

    pub(crate) fn folding_groups(&self) -> Vec<super::Fold> {
        self.browser.folding_groups(&self.browser_hosts(None))
    }

    pub(crate) fn groups_moving(&mut self) -> bool {
        let (browser, hosts) = self.browser_parts(None);
        browser.groups_moving(&hosts)
    }

    pub(crate) fn group_pick(&self, group: GroupId) -> Option<Pick> {
        self.browser.group_pick(group, &self.browser_hosts(None))
    }

    /// The focused workspace's browser tabs and the labels their strips
    /// show, in strip order.
    pub(crate) fn browser_tab_labels(&self, cx: &App) -> Vec<(Pick, SharedString)> {
        self.browser
            .browser_tab_labels(&self.browser_hosts(Some(cx)))
    }

    pub(crate) fn group_lists(&self, group: GroupId, pick: &Pick) -> bool {
        self.browser
            .group_lists(group, pick, &self.browser_hosts(None))
    }

    pub(super) fn workspace_tabs(&self, cx: &App) -> Vec<Pick> {
        self.browser.workspace_tabs(&self.browser_hosts(Some(cx)))
    }

    pub(crate) fn open_workspace_tabs(&self, cx: &App) -> Vec<Pick> {
        self.browser
            .open_workspace_tabs(&self.browser_hosts(Some(cx)))
    }

    pub(crate) fn group_tabs(&self, group: GroupId, cx: &App) -> Vec<Pick> {
        self.browser
            .group_tabs(group, &self.browser_hosts(Some(cx)))
    }

    /// Moves `pick` to `group` and shows it there, as dropping its tab on
    /// the group does, or asking there for a tab a workspace has one of. The
    /// group it leaves goes to the tab after it in its strip, or before it,
    /// and closes when it was its last.
    pub(crate) fn move_tab_to_group(
        &mut self,
        pick: Pick,
        group: GroupId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(source) = self.ensure_layout().map(|layout| layout.owner(&pick)) else {
            return;
        };
        if !self.group_slots().iter().any(|slot| slot.id == group) {
            return;
        }
        if source == group {
            self.show_group_tab(group, pick, window, cx);
            return;
        }
        let before = self.group_tabs(source, cx);
        let shown = self.group_pick(source).as_ref() == Some(&pick);
        self.show_group_tab(group, pick.clone(), window, cx);
        if !shown {
            return;
        }
        if !self.show_neighbour(source, &before, &pick, cx) {
            self.close_group(source, window, cx);
        }
        cx.notify();
    }

    /// Moves the group showing `pick` on to the tab after it in its strip,
    /// or before it, as when the tab is carried off to another window. A
    /// group with no other tab keeps showing it.
    pub(crate) fn leave_tab(&mut self, pick: &Pick, cx: &mut Context<Self>) {
        let Some(source) = self.ensure_layout().map(|layout| layout.owner(pick)) else {
            return;
        };
        if self.group_pick(source).as_ref() != Some(pick) {
            return;
        }
        let before = self.group_tabs(source, cx);
        if self.show_neighbour(source, &before, pick, cx) {
            cx.notify();
        }
    }

    /// Shows in `source` the neighbour of `pick` among `before`, its strip
    /// as it was; whether it had one.
    fn show_neighbour(
        &mut self,
        source: GroupId,
        before: &[Pick],
        pick: &Pick,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(next) = neighbour(before, pick, |_| true) else {
            return false;
        };
        if let Some(layout) = self.ensure_layout() {
            layout.show(source, next);
        }
        self.reconcile_group_terminals(cx);
        self.steer_primary_group(source, cx);
        true
    }

    /// Runs Next Tab, Previous Tab, or a tab by number over the focused
    /// workspace's tabs, its browser tabs included, from the tab the group in
    /// use shows. The daemon knows only its own tabs, so asking it alone
    /// would skip the pages, and would leave a page in front of the Herdr tab
    /// it already focuses. From a terminal to a terminal, the daemon still
    /// moves its focus and the groups follow; a step from or to a page shows
    /// its tab in its own group, as clicking it does. Returns false without a
    /// focused workspace.
    pub(crate) fn step_tab(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(active) = self.ensure_layout().map(|layout| layout.active()) else {
            return false;
        };
        // A focus change in flight lands after this step and would move the
        // group off a page the step showed, so the step is dropped, as one
        // asked of the daemon always was.
        if self.live.activation.fenced() {
            return true;
        }
        let tabs = self.workspace_tabs(cx);
        let current = self.group_pick(active).or_else(|| {
            self.selected_core_group()
                .is_none()
                .then(|| self.focused_herdr_tab())
                .flatten()
                .map(|tab| Pick::Herdr(tab.to_owned()))
        });
        let Some(target) = stepped(&tabs, current.as_ref(), command).cloned() else {
            return true;
        };
        match target {
            Pick::Herdr(tab) if !matches!(current, Some(Pick::Page(_))) => {
                self.navigate(NavigationTarget::Tab(&tab), cx);
                window.focus(&self.focus, cx);
            }
            target => {
                if let Some(group) = self.layout().map(|layout| layout.owner(&target)) {
                    self.show_group_tab(group, target, window, cx);
                }
            }
        }
        true
    }

    /// Shows `pick` in `group`, which owns it.
    fn show_group_tab(
        &mut self,
        group: GroupId,
        pick: Pick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match pick {
            Pick::Herdr(tab) => self.choose_herdr_tab(group, &tab, window, cx),
            Pick::Page(id) => self.show_browser_tab_in(Some(group), id, window, cx),
        }
    }

    /// Steers a group's existing primary connection after replacing its tab,
    /// without activating the group or taking keyboard focus from a page.
    pub(super) fn steer_primary_group(&mut self, group: GroupId, cx: &mut Context<Self>) {
        if Some(group) != self.primary_group() {
            return;
        }
        if let Some(Pick::Herdr(tab)) = self.group_pick(group)
            && self.focused_herdr_tab() != Some(tab.as_str())
        {
            if self.input_ready() && self.menu.page.is_none() {
                self.navigate(NavigationTarget::Tab(&tab), cx);
            } else {
                self.pending_navigation = Some(NavigationTarget::Tab(tab));
            }
        }
    }

    /// What `group` draws. A page another window closed reads as nothing
    /// until the next tick forgets it.
    pub(crate) fn group_shown(&self, group: GroupId, cx: &App) -> Shown {
        // Core needs an exact available group. A legacy window can still draw
        // its terminal while no workspace is focused during connection.
        if self.selected_core_group().is_some() && self.active_browser_key(cx).is_none() {
            return Shown::Empty;
        }
        let Some(layout) = self.layout() else {
            return if self.selected_core_group().is_some() {
                Shown::Empty
            } else {
                Shown::Terminal
            };
        };
        if self.selected_core_group().is_some() && layout.pick(group, None).is_none() {
            return Shown::Empty;
        }
        let focused = |group| self.group_focused_tab(group);
        let shown = layout.shown(group, focused);
        match &shown {
            Shown::Page(id)
                if store(cx)
                    .and_then(|store| store.get(*id))
                    .is_none_or(|tab| {
                        (!self.core_enabled() && tab.requires_core())
                            || self.active_browser_key(cx).as_ref()
                                != Some(&(tab.scope.clone(), tab.workspace_id.clone()))
                    }) =>
            {
                Shown::Empty
            }
            // The window's connection is on its way to the tab its group
            // picked: the terminal keeps its frame until the daemon focuses
            // the tab, rather than flashing a stand-in for one round trip.
            Shown::Elsewhere(tab)
                if Some(group) == self.primary_group() && !self.parked_focuses(tab) =>
            {
                Shown::Terminal
            }
            _ => shown,
        }
    }

    /// Settles which group holds the window's connection, then, when that
    /// is `group`, focuses the Herdr tab it picked if the connection shows
    /// another, and hands it the keyboard. A group with a parked connection
    /// steers that one to its tab instead.
    fn bring_group_tab(&mut self, group: GroupId, window: &mut Window, cx: &mut Context<Self>) {
        self.reconcile_group_terminals(cx);
        if self.primary_group() != Some(group) {
            return;
        }
        // This choice supersedes any deferred target, including when its
        // terminal is already focused and needs no new request.
        self.pending_navigation = None;
        self.steer_primary_group(group, cx);
        if matches!(self.group_pick(group), Some(Pick::Herdr(_)) | None) {
            window.focus(&self.focus, cx);
        }
    }

    /// Makes `group` the one in use, which brings it the tab it picked.
    pub(crate) fn activate_group(
        &mut self,
        group: GroupId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.ensure_layout() else {
            return;
        };
        if !layout.activate(group) {
            return;
        }
        self.bring_group_tab(group, window, cx);
        cx.notify();
    }

    /// Shows Herdr tab `tab` in `group`.
    pub(crate) fn choose_herdr_tab(
        &mut self,
        group: GroupId,
        tab: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.ensure_layout() else {
            return;
        };
        layout.choose(group, Pick::Herdr(tab.to_owned()));
        self.bring_group_tab(group, window, cx);
        cx.notify();
    }

    pub(crate) fn expect_new_tab_in(&mut self, group: GroupId) {
        let (browser, hosts) = self.browser_parts(None);
        browser.expect_new_tab_in(group, &hosts);
    }

    #[cfg(test)]
    pub(crate) fn expected_new_tab_group(&self) -> Option<GroupId> {
        self.browser.expected_new_tab_group()
    }

    /// Follows the window's connection moving its focus, from a shortcut, an
    /// agent, a closed tab, or a group's "+". The tab's own group shows it
    /// and takes the connection; a new tab goes to the group whose "+" asked
    /// for it, or else to the group in use. The group that had the
    /// connection gets a parked one for its own tab. Arriving in another
    /// workspace moves its group only off a Herdr tab: one showing a page
    /// keeps it.
    pub(crate) fn terminal_focus_moved(
        &mut self,
        new: &str,
        arrived: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(key) = self.browser_key() else {
            return;
        };
        // Restore a saved layout before reading its groups.
        self.layout_for(key.clone());
        let asked = self
            .browser
            .new_tab_group
            .take_if(|(asked, _)| *asked == key)
            .map(|(_, group)| group);
        let primary = self.primary_group();
        let on_page =
            primary.is_some_and(|group| matches!(self.group_pick(group), Some(Pick::Page(_))));
        // A restored layout keeps its tabs; the group in use is taken back
        // to its own instead.
        let restoring = self.browser.restored.as_ref() == Some(&key);
        if arrived && (on_page || restoring) {
            return;
        }
        // The daemon moved on from a tab it closed. When that lands in
        // another group, the group that showed the closed tab goes to one of
        // its own instead; see `settle_groups`.
        let left_closed = primary
            .and_then(|group| self.group_pick(group))
            .is_some_and(|pick| !self.workspace_tabs(cx).contains(&pick));
        let pick = Pick::Herdr(new.to_owned());
        let layout = self.layout_for(key);
        let split = layout.len() > 1;
        let target = asked.or_else(|| {
            if !split {
                return None;
            }
            Some(layout.claimed(&pick).unwrap_or(layout.active()))
        });
        // The group holding the connection keeps it for a tab of its own,
        // without becoming the group in use: an agent must not take that
        // from a group showing a tab. A group in use that shows none has
        // nothing to keep, and Close Tab there would close the group rather
        // than the tab the focus just moved to.
        let idle = layout.pick(layout.active(), None).is_none();
        let own = target.is_some()
            && target == primary
            && !idle
            && primary.is_some_and(|primary| layout.pick(primary, None).is_some());
        match (target, primary) {
            (Some(group), Some(primary)) if group != primary && left_closed && asked.is_none() => {}
            (Some(group), _) if !own => {
                layout.choose(group, pick);
                self.set_primary_group(group);
            }
            (_, Some(primary)) => layout.focus_moved(primary, new),
            (_, None) => {}
        }
        self.reconcile_group_terminals(cx);
        cx.notify();
    }

    /// Shows a browser tab in its group, or in the group in use when it is
    /// new.
    pub(crate) fn show_browser_tab(
        &mut self,
        id: TabId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_browser_tab_in(None, id, window, cx);
    }

    /// Shows a browser tab in `group`, which takes it, or else in the group
    /// it belongs to: the group in use of its workspace when it is new.
    pub(crate) fn show_browser_tab_in(
        &mut self,
        group: Option<GroupId>,
        id: TabId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = store(cx).and_then(|store| store.get(id)).cloned() else {
            return;
        };
        if !self.core_enabled() && tab.requires_core() {
            self.show_flash(
                crate::window::Flash::warning(crate::core_feature::disabled_message()),
                cx,
            );
            return;
        }
        if tab.core_group().is_some()
            && self.active_browser_key(cx).as_ref()
                != Some(&(tab.scope.clone(), tab.workspace_id.clone()))
        {
            return;
        }
        let layout = self.layout_for((tab.scope.clone(), tab.workspace_id.clone()));
        let group = group.unwrap_or_else(|| layout.owner(&Pick::Page(id)));
        layout.choose(group, Pick::Page(id));
        // A page chosen in the primary group replaces its pending terminal
        // choice. Pages in other groups leave that navigation alone.
        if self.primary_group() == Some(group) {
            self.pending_navigation = None;
        }
        if let Some(Location::CoreTerminal { .. }) = &tab.location {
            self.focus_core_terminal(id, window, cx);
        }
        cx.notify();
    }

    /// Closes a browser tab. The group showing it moves to the tab after it
    /// in its strip, or before it; a group of a split left with none closes.
    pub(crate) fn close_browser_tab(
        &mut self,
        id: TabId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = store(cx).and_then(|store| store.get(id)).cloned() else {
            return;
        };
        let pick = Pick::Page(id);
        let key = (tab.scope.clone(), tab.workspace_id.clone());
        // Only the focused workspace's strips are known here; another
        // workspace's group is given a tab when it is next shown.
        let focused = self.active_browser_key(cx).as_ref() == Some(&key);
        let group = self
            .browser
            .layouts
            .get(&key)
            .filter(|_| focused)
            .map(|layout| layout.owner(&pick));
        let before = group.map_or_else(Vec::new, |group| self.group_tabs(group, cx));
        let shown = self
            .browser
            .layouts
            .get_mut(&key)
            .and_then(|layout| layout.forget(&pick));
        crate::browser::Store::update(cx, |store| store.close(id));
        self.forget_browser_tabs(|tab| tab == id);
        if !focused {
            self.browser.emptied.extend(shown);
        }
        if let (Some(group), Some(shown)) = (group, shown)
            && group == shown
        {
            // Alone, a group moves among its browser tabs and otherwise
            // follows the terminal again, wherever the daemon's focus is.
            let split = self.is_split();
            let near = |tab: &Pick| split || matches!(tab, Pick::Page(_));
            match neighbour(&before, &pick, near) {
                Some(next) => self.show_group_tab(group, next, window, cx),
                None if split => self.close_group(group, window, cx),
                None => {}
            }
        }
        cx.notify();
    }

    /// Splits `group`: a new group opens to its right without tabs, and
    /// every tab stays where it is.
    pub(crate) fn split_group(
        &mut self,
        group: GroupId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new = self.browser.group_ids.next();
        let focused = self
            .selected_core_group()
            .is_none()
            .then(|| self.focused_herdr_tab())
            .flatten()
            .map(str::to_owned);
        let tabs = self.workspace_tabs(cx);
        // The group holding the window's connection keeps it: the new group
        // has no tab to show yet.
        let primary = self.primary_group();
        let Some(layout) = self.ensure_layout() else {
            return;
        };
        if !layout.split(group, new, focused.as_deref(), tabs) {
            return;
        }
        if let Some(primary) = primary {
            self.set_primary_group(primary);
        }
        self.browser
            .group_motion
            .open(new, group, crate::motion::now());
        self.bring_group_tab(new, window, cx);
        cx.notify();
    }

    /// Splits the group in use.
    pub(crate) fn split_active_group(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(group) = self.ensure_layout().map(|layout| layout.active()) {
            self.split_group(group, window, cx);
        }
    }

    /// Closes `group`; its tabs go to the group that takes its room.
    pub(crate) fn close_group(
        &mut self,
        group: GroupId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.ensure_layout() else {
            return;
        };
        let target = layout.fold_target(group);
        if !layout.close(group) {
            return;
        }
        let active = layout.active();
        if let Some((index, share, into)) = target {
            self.browser
                .group_motion
                .fold(group, index, share, into, crate::motion::now());
        }
        self.forget_group_terminal(group);
        self.bring_group_tab(active, window, cx);
        cx.notify();
    }

    pub(crate) fn drag_divider(&mut self, divider: usize, offset: f32, width: f32) -> bool {
        let (browser, hosts) = self.browser_parts(None);
        browser.drag_divider(divider, offset, width, &hosts)
    }
}
