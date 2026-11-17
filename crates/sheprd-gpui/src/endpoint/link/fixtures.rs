// ABOUTME: Installs link state for headless fixtures without dialing fixture targets.
// ABOUTME: Routes fixture writes through the same endpoint transition owner.
use super::*;

impl Endpoint {
    pub(crate) fn never_connect(&mut self) {
        self.set_retry_at_for_test(Instant::now() + Duration::from_secs(3600));
    }

    pub(crate) fn set_retry_at_for_test(&mut self, at: Instant) {
        self.enter(Transition::RetryAt(at));
    }

    pub(crate) fn lose_connection_for_test(&mut self, reason: &str) {
        self.set_failure_for_test(Some(Failure::Transport {
            reason: Some(reason.into()),
            needs_user: false,
        }));
    }

    pub(crate) fn lose_herdr_for_test(&mut self, reason: &str) {
        self.set_failure_for_test(Some(Failure::Missing {
            reason: Some(reason.into()),
        }));
    }

    pub(crate) fn set_version_mismatch_for_test(&mut self, mismatch: Option<VersionMismatch>) {
        self.set_failure_for_test(mismatch.map(|mismatch| Failure::Version {
            mismatch,
            reason: None,
        }));
    }

    pub(crate) fn clear_outage_for_test(&mut self) {
        self.set_failure_for_test(None);
    }

    fn set_failure_for_test(&mut self, why: Option<Failure>) {
        let retry_at = self.retry_at();
        self.enter(Transition::Fixture(if self.connection.handle.is_some() {
            Link::Dialing { retry_at, why }
        } else {
            Link::Waiting { retry_at, why }
        }));
    }

    pub(crate) fn detach_for_test(&mut self) {
        self.enter(Transition::Fixture(Link::Detached {
            retry_at: self.retry_at(),
            why: None,
        }));
    }

    pub(crate) fn set_enabled_for_test(&mut self, enabled: bool) {
        self.enter(Transition::Fixture(Link::initial(enabled)));
        self.enter(Transition::Observe(Instant::now()));
    }

    pub(in crate::endpoint) fn online_since(&self) -> Option<Instant> {
        match self.link {
            Link::Up { since, .. } => since,
            _ => None,
        }
    }

    pub(in crate::endpoint) fn clear_stability_for_test(&mut self) {
        if let Link::Up { retry_at, .. } = self.link {
            self.enter(Transition::Fixture(Link::Up {
                retry_at,
                since: None,
            }));
        }
    }
}
