// ABOUTME: Owns typed window and connection notices with bounded display lifetimes.
// ABOUTME: Fences held-input restoration against newer notices and explicit clears.

use std::time::{Duration, Instant};

const OPERATION_DURATION: Duration = Duration::from_secs(10);

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
struct Notice {
    text: String,
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
    revision: u64,
}

pub(crate) struct HeldNotice {
    notice: Option<Notice>,
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

    pub(crate) fn clear(&mut self) {
        self.transition(Transition::Clear);
    }

    pub(crate) fn take(&mut self) -> HeldNotice {
        let notice = self.transition(Transition::Take);
        HeldNotice {
            notice,
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
            .filter(|notice| notice.active(now))
            .map(|notice| notice.text.as_str())
    }

    pub(crate) fn is_some_at(&self, now: Instant) -> bool {
        self.text_at(now).is_some()
    }

    pub(crate) fn connection_reason(&self) -> Option<&str> {
        self.current
            .as_ref()
            .filter(|notice| notice.cause == Cause::Connection)
            .map(|notice| notice.text.as_str())
    }

    pub(crate) fn is_some(&self) -> bool {
        self.text().is_some()
    }
    pub(crate) fn is_none(&self) -> bool {
        self.text().is_none()
    }

    /// Every mutation passes here, including temporary removal during input replay.
    fn transition(&mut self, event: Transition) -> Option<Notice> {
        match event {
            Transition::Show { cause, text, now } => {
                if cause != Cause::Connection && self.connection_reason().is_some() {
                    return None;
                }
                self.revision = self.revision.wrapping_add(1);
                self.current.replace(Notice {
                    text,
                    cause,
                    until: (cause != Cause::Connection).then_some(now + OPERATION_DURATION),
                })
            }
            Transition::Clear => {
                self.revision = self.revision.wrapping_add(1);
                self.current.take()
            }
            Transition::Take => self.current.take(),
            Transition::Restore(held) => {
                if held.revision == self.revision && self.current.is_none() {
                    self.current = held.notice.filter(|notice| notice.active(Instant::now()));
                }
                None
            }
            Transition::Expire(now) => {
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
