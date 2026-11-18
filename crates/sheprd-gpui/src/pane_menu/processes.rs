// ABOUTME: Owns the pane process monitor, exact selection, and end-process consent.
// ABOUTME: Cancels monitoring when the active popup owner drops.
//! The pane menu's process list: every process under the pane with its CPU
//! and memory, refreshed while open, and ending the chosen ones after a
//! confirmation. The worker does the listing and signalling; this side only
//! keeps what it last reported and what the user chose.

use super::PaneMenu;
use crate::direction::Logical;
use crate::i18n::fl;
use crate::{
    SheprdWindow,
    menu::Page,
    processes::{Identity, Listing, Outcome, Refusal, Update, Watch},
};
use gpui::{prelude::*, *};
use std::collections::BTreeSet;
mod paint;
mod render;
pub(super) use paint::{Confirmation, Paint};

#[cfg(feature = "integration-test")]
mod native_checks;

pub(crate) struct PaneProcesses {
    /// None once the list stopped updating.
    watch: Option<Watch>,
    listing: Option<Listing>,
    /// Exactly the processes the user chose, pid and start time.
    chosen: BTreeSet<Identity>,
    /// The keyboard's row, kept in view.
    cursor: Option<usize>,
    scroll: ScrollHandle,
    error: Option<String>,
    /// What the last kill did.
    report: Option<String>,
    /// A kill is with the worker.
    ending: bool,
}

impl PaneProcesses {
    pub(super) fn new(watch: Watch) -> Self {
        Self {
            watch: Some(watch),
            listing: None,
            chosen: BTreeSet::new(),
            cursor: None,
            scroll: ScrollHandle::new(),
            error: None,
            report: None,
            ending: false,
        }
    }

    fn stop(&mut self, error: &crate::Error) {
        self.watch = None;
        self.ending = false;
        self.error = Some(describe(error));
    }

    fn apply(&mut self, update: Update) {
        match update {
            Update::Listing(Ok(listing)) => {
                // A chosen process that exited, or whose pid now names
                // another, is no longer offered for ending.
                self.chosen.retain(|identity| listing.contains(*identity));
                self.cursor = self
                    .cursor
                    .map(|cursor| cursor.min(listing.rows.len().saturating_sub(1)));
                self.listing = Some(listing);
                self.error = None;
            }
            Update::Listing(Err(error)) => self.error = Some(describe(&error)),
            Update::Killed(outcomes) => {
                self.ending = false;
                for (identity, outcome) in &outcomes {
                    if *outcome == Outcome::Signalled {
                        self.chosen.remove(identity);
                    }
                }
                self.report = Some(report(&outcomes));
            }
        }
    }

    /// Whether `identity` can be chosen: listed, and not the pane's own process.
    fn choosable(&self, identity: Identity) -> bool {
        self.listing
            .as_ref()
            .is_some_and(|listing| listing.root() != Some(identity) && listing.contains(identity))
    }

    fn toggle(&mut self, identity: Identity) {
        if !self.chosen.remove(&identity) && self.choosable(identity) {
            self.chosen.insert(identity);
        }
    }

    /// The chosen rows, in the tree's order, for the confirmation.
    fn chosen_rows(&self) -> impl Iterator<Item = &crate::processes::Row> {
        self.listing
            .iter()
            .flat_map(|listing| &listing.rows)
            .filter(|row| self.chosen.contains(&row.entry.identity))
    }
}

/// The display boundary: the message, then each cause.
fn describe(error: &crate::Error) -> String {
    let mut text = error.to_string();
    let mut source = std::error::Error::source(error);
    while let Some(cause) = source {
        text.push(' ');
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

fn report(outcomes: &[(Identity, Outcome)]) -> String {
    let count = |wanted: fn(&Outcome) -> bool| outcomes.iter().filter(|(_, o)| wanted(o)).count();
    let mut parts = Vec::new();
    let signalled = count(|o| *o == Outcome::Signalled);
    if signalled > 0 {
        parts.push(fl!("pane-menu-processes-signalled", count = signalled));
    }
    let exited = count(|o| *o == Outcome::Refused(Refusal::Exited));
    if exited > 0 {
        parts.push(fl!("pane-menu-processes-exited", count = exited));
    }
    let moved = count(|o| {
        matches!(
            o,
            Outcome::Refused(Refusal::Elsewhere | Refusal::Root | Refusal::RootExited)
        )
    });
    if moved > 0 {
        parts.push(fl!("pane-menu-processes-moved", count = moved));
    }
    let failed = count(|o| *o == Outcome::Failed);
    if failed > 0 {
        parts.push(fl!("pane-menu-processes-failed", count = failed));
    }
    parts.join(" ")
}

/// Share of one core: whole percents once they are large.
pub(super) fn cpu_text(cpu: Option<f32>) -> String {
    match cpu {
        None => "–".into(),
        Some(cpu) if cpu < 9.95 => format!("{cpu:.1}%"),
        Some(cpu) => format!("{cpu:.0}%"),
    }
}

pub(super) fn memory_text(bytes: u64) -> String {
    const MIB: f64 = (1u64 << 20) as f64;
    let mib = bytes as f64 / MIB;
    if mib >= 1024. {
        fl!(
            "pane-menu-processes-gigabytes",
            size = format!("{:.1}", mib / 1024.)
        )
    } else if mib >= 10. {
        fl!("pane-menu-processes-megabytes", size = format!("{mib:.0}"))
    } else {
        fl!("pane-menu-processes-megabytes", size = format!("{mib:.1}"))
    }
}

impl SheprdWindow {
    pub(super) fn open_pane_processes(&mut self, cx: &mut Context<Self>) {
        let Some(pane) = &mut self.menu.pane else {
            return;
        };
        let Some(daemon) = pane.daemon.clone() else {
            return;
        };
        match Watch::start(daemon, pane.target.pane.clone()) {
            Ok(watch) => self.show_pane_processes(watch, cx),
            Err(error) => {
                pane.error = Some(describe(&error));
                cx.notify();
            }
        }
    }

    pub(super) fn show_pane_processes(&mut self, watch: Watch, cx: &mut Context<Self>) {
        let Some(pane) = &mut self.menu.pane else {
            return;
        };
        pane.processes = Some(PaneProcesses::new(watch));
        pane.error = None;
        self.menu.page = Some(Page::PaneProcesses);
        cx.notify();
    }

    /// Takes the worker's answers, and stops it once the pane or the
    /// connection it belongs to is gone.
    pub(crate) fn poll_pane_processes(&mut self, cx: &mut Context<Self>) {
        if self
            .menu
            .pane
            .as_ref()
            .and_then(|pane| pane.processes.as_ref())
            .is_none_or(|processes| processes.watch.is_none())
        {
            return;
        }
        let current = self.validate_pane_target().map(drop);
        let Some(processes) = self
            .menu
            .pane
            .as_mut()
            .and_then(|pane| pane.processes.as_mut())
        else {
            return;
        };
        if let Err(error) = current {
            processes.stop(&error);
            self.leave_kill_confirmation();
            cx.notify();
            return;
        }
        let mut changed = false;
        while let Some(watch) = &processes.watch {
            match watch.try_update() {
                Ok(Some(update)) => {
                    let killed = matches!(update, Update::Killed(_));
                    processes.apply(update);
                    changed = true;
                    if killed {
                        self.leave_kill_confirmation();
                        break;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    processes.stop(&error);
                    changed = true;
                }
            }
        }
        if changed {
            cx.notify();
        }
    }

    fn leave_kill_confirmation(&mut self) {
        if self.menu.page == Some(Page::KillProcesses) {
            self.menu.page = Some(Page::PaneProcesses);
        }
    }

    fn pane_processes(&mut self) -> Option<&mut PaneProcesses> {
        self.menu.pane.as_mut()?.processes.as_mut()
    }

    fn toggle_pane_process(&mut self, identity: Identity, cx: &mut Context<Self>) {
        if let Some(processes) = self.pane_processes() {
            processes.toggle(identity);
            cx.notify();
        }
    }

    fn confirm_pane_processes(&mut self, cx: &mut Context<Self>) {
        if self
            .pane_processes()
            .is_some_and(|processes| !processes.chosen.is_empty() && !processes.ending)
        {
            self.menu.page = Some(Page::KillProcesses);
            cx.notify();
        }
    }

    fn end_pane_processes(&mut self, cx: &mut Context<Self>) {
        let current = self.validate_pane_target().map(drop);
        let Some(processes) = self.pane_processes() else {
            return;
        };
        if processes.ending || processes.chosen.is_empty() {
            return;
        }
        let sent = current.and_then(|()| {
            processes
                .watch
                .as_ref()
                .ok_or(crate::processes::Error::Stopped)?
                .kill(processes.chosen.iter().copied().collect())
        });
        match sent {
            Ok(()) => {
                processes.ending = true;
                processes.report = None;
            }
            Err(error) => processes.error = Some(describe(&error)),
        }
        self.leave_kill_confirmation();
        cx.notify();
    }

    /// Keys on the process pages; false for keys they leave alone.
    pub(super) fn pane_processes_key(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let confirming = self.menu.page == Some(Page::KillProcesses);
        let Some(processes) = self.pane_processes() else {
            return false;
        };
        match key {
            "escape" if confirming => self.leave_kill_confirmation(),
            "enter" if confirming && event.is_held => {}
            "enter" if confirming => self.end_pane_processes(cx),
            "enter" => self.confirm_pane_processes(cx),
            "up" | "down" if !confirming => {
                let count = processes.listing.as_ref().map_or(0, |l| l.rows.len());
                if count == 0 {
                    return true;
                }
                processes.cursor = Some(match (processes.cursor, key) {
                    (None, "up") => count - 1,
                    (None, _) => 0,
                    (Some(i), "up") => (i + count - 1) % count,
                    (Some(i), _) => (i + 1) % count,
                });
                if let Some(cursor) = processes.cursor {
                    processes.scroll.scroll_to_item(cursor);
                }
            }
            "space" if !confirming => {
                if let Some(identity) = processes.cursor.and_then(|cursor| {
                    processes
                        .listing
                        .as_ref()?
                        .rows
                        .get(cursor)
                        .map(|row| row.entry.identity)
                }) {
                    processes.toggle(identity);
                }
            }
            _ => return false,
        }
        cx.notify();
        true
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) use tests::departure::fixture as departure_fixture;
