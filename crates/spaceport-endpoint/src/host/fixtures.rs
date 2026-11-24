//! Installs link state for the app's headless fixtures without dialing them.
//! Every write is a transition, so a fixture cannot leave the link machine in a
//! state no event produces.
use super::link::{Failure, Link};
use super::{Host, Transport};
use std::time::{Duration, Instant};

impl Host {
    pub fn set_retry_at_for_test(&mut self, at: Instant) {
        let _ = self.enter(super::Transition::RetryAt(at));
    }

    /// The failure a fixture installs, on the attempt `handled` describes: a
    /// dialing link while a handle exists, a waiting one otherwise.
    pub fn set_failure_for_test(&mut self, handled: bool, why: Option<Failure>) {
        let retry_at = self.retry_at();
        let link = if handled {
            Link::Dialing { retry_at, why }
        } else {
            Link::Waiting { retry_at, why }
        };
        let _ = self.enter(super::Transition::Fixture(link));
    }

    pub fn detach_for_test(&mut self) {
        let _ = self.enter(super::Transition::Fixture(Link::Detached {
            retry_at: self.retry_at(),
            why: None,
        }));
    }

    pub fn set_enabled_for_test(&mut self, enabled: bool, transport: Transport) {
        let _ = self.enter(super::Transition::Fixture(Link::initial(enabled)));
        let _ = self.enter(super::Transition::Observe {
            now: Instant::now(),
            transport,
        });
    }

    /// The attempt count the retry curve is built from.
    pub fn attempts_for_test(&self) -> u32 {
        self.attempts
    }

    pub fn set_attempts_for_test(&mut self, attempts: u32) {
        self.attempts = attempts;
    }

    /// When this host became a serving link, if it is one.
    pub fn online_since(&self) -> Option<Instant> {
        match self.link {
            Link::Up { since, .. } => since,
            _ => None,
        }
    }

    pub fn clear_stability_for_test(&mut self) {
        if let Link::Up { retry_at, .. } = self.link {
            let _ = self.enter(super::Transition::Fixture(Link::Up {
                retry_at,
                since: None,
            }));
        }
    }

    /// A host that will not dial for an hour, for a fixture that only wants a
    /// settled link.
    pub fn never_connect_for_test(&mut self) {
        self.set_retry_at_for_test(Instant::now() + Duration::from_secs(3600));
    }
}
