// ABOUTME: Owns typed window and connection notices with bounded display lifetimes.
// ABOUTME: Fences held-input restoration against newer notices and explicit clears.

use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

const OPERATION_DURATION: Duration = Duration::from_secs(10);
const HISTORY_LIMIT: usize = 32;
const TEXT_LIMIT: usize = 4096;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cause {
    Activation,
    Command,
    Configuration,
    Connection,
    Daemon,
    Input,
    Transfer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Notice {
    pub(crate) id: u64,
    pub(crate) text: Arc<str>,
    cause: Cause,
    until: Option<Instant>,
}

impl Notice {
    fn active(&self, now: Instant) -> bool {
        self.until.is_none_or(|until| now < until)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Notices {
    current: Option<Notice>,
    history: VecDeque<Notice>,
    paused: Option<Instant>,
    revision: u64,
}

pub(crate) struct HeldNotice {
    notice: Option<Notice>,
    paused: Option<Instant>,
    revision: u64,
}

enum Transition {
    Show {
        cause: Cause,
        text: String,
        now: Instant,
    },
    Clear,
    Take,
    Restore(HeldNotice),
    Expire(Instant),
}

impl Notices {
    pub(crate) fn show(&mut self, cause: Cause, text: String) {
        self.transition(Transition::Show {
            cause,
            text,
            now: Instant::now(),
        });
    }

    pub(crate) fn record(&mut self, cause: Cause, text: String) {
        self.record_at(cause, text, Instant::now());
    }

    fn record_at(&mut self, cause: Cause, mut text: String, now: Instant) -> Notice {
        if text.len() > TEXT_LIMIT {
            text.truncate(text.floor_char_boundary(TEXT_LIMIT - '…'.len_utf8()));
            text.push('…');
        }
        let notice = Notice {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            text: text.into(),
            cause,
            until: (cause != Cause::Connection).then_some(now + OPERATION_DURATION),
        };
        self.history.push_back(notice.clone());
        if self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
        notice
    }

    pub(crate) fn clear(&mut self) {
        self.transition(Transition::Clear);
    }

    pub(crate) fn take(&mut self) -> HeldNotice {
        let paused = self.paused;
        let notice = self.transition(Transition::Take);
        HeldNotice {
            notice,
            paused,
            revision: self.revision,
        }
    }

    pub(crate) fn restore(&mut self, held: HeldNotice) {
        self.transition(Transition::Restore(held));
    }

    pub(crate) fn expire(&mut self, now: Instant) -> bool {
        self.transition(Transition::Expire(now)).is_some()
    }

    pub(crate) fn text(&self) -> Option<&str> {
        self.text_at(Instant::now())
    }

    /// The current notice as it reads at `now`: a caller that already holds an
    /// instant, such as the frame it is drawing, passes it rather than letting
    /// a second clock read decide.
    pub(crate) fn text_at(&self, now: Instant) -> Option<&str> {
        self.current
            .as_ref()
            .filter(|notice| notice.active(self.paused.unwrap_or(now)))
            .map(|notice| notice.text.as_ref())
    }

    pub(crate) fn history(&self) -> impl DoubleEndedIterator<Item = &Notice> {
        self.history.iter()
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused.is_some()
    }

    pub(crate) fn inherit_display(&mut self, previous: &Self) {
        if let (Some(current), Some(old)) = (&mut self.current, &previous.current)
            && current.id == old.id
        {
            current.until = old.until;
            self.paused = previous.paused;
        }
    }

    pub(crate) fn pause(&mut self, paused: bool, now: Instant) {
        if paused {
            if self.paused.is_none()
                && self
                    .current
                    .as_ref()
                    .is_some_and(|notice| notice.active(now))
            {
                self.paused = Some(now);
            }
        } else if let Some(since) = self.paused.take()
            && let Some(until) = self
                .current
                .as_mut()
                .and_then(|notice| notice.until.as_mut())
        {
            *until += now.saturating_duration_since(since);
        }
    }

    pub(crate) fn is_some_at(&self, now: Instant) -> bool {
        self.text_at(now).is_some()
    }

    pub(crate) fn connection_reason(&self) -> Option<&str> {
        self.current
            .as_ref()
            .filter(|notice| notice.cause == Cause::Connection)
            .map(|notice| notice.text.as_ref())
    }

    /// Only harnesses ask "any notice at all": the app reads
    /// [`Self::is_some_at`] at the instant it draws.
    #[cfg(any(test, feature = "integration-test"))]
    pub(crate) fn is_some(&self) -> bool {
        self.text().is_some()
    }

    pub(crate) fn is_none(&self) -> bool {
        self.text().is_none()
    }

    /// Replaces, removes, and restores the current notice, including held input replay.
    fn transition(&mut self, event: Transition) -> Option<Notice> {
        match event {
            Transition::Show { cause, text, now } => {
                let notice = self.record_at(cause, text, now);
                if cause != Cause::Connection && self.connection_reason().is_some() {
                    return None;
                }
                self.paused = self.paused.map(|_| now);
                self.revision = self.revision.wrapping_add(1);
                self.current.replace(notice)
            }
            Transition::Clear => {
                self.paused = None;
                self.revision = self.revision.wrapping_add(1);
                self.current.take()
            }
            Transition::Take => {
                self.paused = None;
                self.current.take()
            }
            Transition::Restore(held) => {
                if held.revision == self.revision && self.current.is_none() {
                    self.current = held
                        .notice
                        .filter(|notice| notice.active(held.paused.unwrap_or_else(Instant::now)));
                    self.paused = self.current.as_ref().and(held.paused);
                }
                None
            }
            Transition::Expire(now) => {
                if self.paused.is_some() {
                    return None;
                }
                if self
                    .current
                    .as_ref()
                    .is_some_and(|notice| !notice.active(now))
                {
                    self.revision = self.revision.wrapping_add(1);
                    self.current.take()
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
