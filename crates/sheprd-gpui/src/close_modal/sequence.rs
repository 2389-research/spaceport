// ABOUTME: Runs an exactly captured tab batch after one complete confirmation.
// ABOUTME: Waits for terminal removal and surface barriers without replaying sent closes.

use super::{
    CloseConfirmation, Target,
    batch::{Batch, Inspection, Item},
};
use crate::{
    SheprdWindow,
    browser::Pick,
    i18n::{Localize, fl},
};
use gpui::{Context, Window};
use std::time::{Duration, Instant};

/// How long a close the daemon was sent may go unanswered before the run
/// gives up rather than wait on a tab that is not going away.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) struct CloseRun {
    batch: Batch,
    next: usize,
    phase: Phase,
}

#[derive(Clone, Copy)]
enum Phase {
    Ready(Instant),
    Sent(Instant),
    Settling(Instant),
}

impl SheprdWindow {
    /// Closes `tabs` of the focused workspace in order.
    pub(crate) fn close_tabs(
        &mut self,
        tabs: Vec<Pick>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if tabs.is_empty() {
            return;
        }
        if self.browser.closing.is_some() {
            self.notices.show(
                crate::notice::Cause::Command,
                fl!("close-modal-batch-in-progress"),
            );
            return;
        }
        let batch = match Batch::capture(self, tabs, cx) {
            Ok(batch) => batch,
            Err(error) => {
                self.notices
                    .show(crate::notice::Cause::Command, error.localize());
                return;
            }
        };
        let label = batch.labels();
        self.show_close(
            Some(CloseConfirmation {
                target: Target::Batch(batch),
                label,
                interrupts: None,
                confirm_selected: false,
                do_not_ask_again: false,
                error: None,
            }),
            window,
            cx,
        );
    }

    pub(super) fn begin_close_batch(
        &mut self,
        batch: Batch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let result = batch
            .validate(self, &batch.items, cx)
            .and_then(|inspection| {
                if matches!(inspection, Inspection::Busy)
                    || (batch.items.iter().any(|item| item.tab().is_some()) && !self.input_ready())
                {
                    Err(crate::Error::ConnectionNotReady)
                } else {
                    Ok(())
                }
            });
        if let Err(error) = result {
            if let Some(close) = &mut self.menu.close {
                close.error = Some(error.localize());
            }
            cx.notify();
            return;
        }
        self.dismiss_menu(window, cx);
        if self.menu.page.is_some() {
            return;
        }
        self.browser.closing = Some(CloseRun {
            batch,
            next: 0,
            phase: Phase::Ready(Instant::now()),
        });
        self.advance_close_run(window, cx);
    }

    /// Displays partial completion without retrying or rolling back any sent close.
    fn stopped_close_batch(&mut self, run: &CloseRun, reason: String) {
        let mut text = fl!(
            "close-modal-batch-stopped",
            completed = run.next,
            total = run.batch.items.len(),
            error = reason
        );
        if matches!(run.phase, Phase::Sent(_)) {
            text.push('\n');
            text.push_str(&fl!("close-modal-batch-pending"));
        }
        self.notices.show(crate::notice::Cause::Command, text);
    }

    /// Takes the run as far as it can go without the user or the daemon: on
    /// every tick, and when it starts.
    pub(crate) fn advance_close_run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.advance_close_run_at(window, cx, Instant::now());
    }

    fn advance_close_run_at(&mut self, window: &mut Window, cx: &mut Context<Self>, now: Instant) {
        let Some(mut run) = self.browser.closing.take() else {
            return;
        };
        loop {
            let items = if matches!(run.phase, Phase::Ready(_)) {
                &run.batch.items[run.next..]
            } else {
                &[][..]
            };
            match run.batch.validate(self, items, cx) {
                Ok(Inspection::Current) => {}
                Ok(Inspection::Busy) => {
                    let since = match run.phase {
                        Phase::Ready(at) | Phase::Sent(at) | Phase::Settling(at) => at,
                    };
                    if now.saturating_duration_since(since) < ANSWER_TIMEOUT {
                        break;
                    }
                    self.stopped_close_batch(&run, crate::Error::ConnectionNotReady.localize());
                    return;
                }
                Err(error) => {
                    self.stopped_close_batch(&run, error.localize());
                    return;
                }
            }
            if let Phase::Settling(since) = run.phase {
                if self.input_ready()
                    || run.batch.items[run.next..]
                        .iter()
                        .all(|item| item.tab().is_none())
                {
                    run.phase = Phase::Ready(now);
                    continue;
                } else if now.saturating_duration_since(since) >= ANSWER_TIMEOUT {
                    self.stopped_close_batch(&run, fl!("close-modal-batch-surface-timeout"));
                    return;
                } else {
                    break;
                }
            }
            let Some(item) = run.batch.items.get(run.next).cloned() else {
                return;
            };
            if let Phase::Sent(sent) = run.phase {
                let Some(tab) = item.tab() else {
                    return;
                };
                let open = self.live.snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot
                        .tabs
                        .iter()
                        .any(|candidate| candidate.tab_id == tab)
                });
                if !open {
                    run.next += 1;
                    run.phase = Phase::Settling(sent);
                    continue;
                }
                if now.saturating_duration_since(sent) < ANSWER_TIMEOUT {
                    break;
                }
                self.stopped_close_batch(&run, fl!("close-modal-batch-timeout"));
                return;
            }
            // Removing a tab from the snapshot does not acknowledge its
            // close's surface barrier. An immediate close would fail its
            // readiness check and leave an error dialog instead of waiting.
            if item.tab().is_some() && !self.input_ready() {
                if let Phase::Ready(since) = run.phase
                    && now.saturating_duration_since(since) >= ANSWER_TIMEOUT
                {
                    self.stopped_close_batch(&run, fl!("close-modal-batch-surface-timeout"));
                    return;
                }
                break;
            }
            match item {
                Item::View(tab) => {
                    self.close_browser_tab(tab.id, window, cx);
                    run.next += 1;
                }
                Item::Herdr { close, .. } => {
                    let Target::Herdr(target) = close.target else {
                        return;
                    };
                    if let Err(error) = self.queue_herdr_close(&target, cx) {
                        self.stopped_close_batch(&run, error.localize());
                        return;
                    }
                    run.phase = Phase::Sent(now);
                    break;
                }
            }
        }
        self.browser.closing = Some(run);
    }
}

#[cfg(test)]
mod tests;
