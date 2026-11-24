// ABOUTME: Installs link state for headless fixtures without dialing fixture targets.
// ABOUTME: Routes fixture writes through the crate's transition owner and the app's performers.
use super::*;

impl Endpoint {
    pub(crate) fn never_connect(&mut self) {
        self.host.never_connect_for_test();
    }

    pub(crate) fn set_retry_at_for_test(&mut self, at: Instant) {
        self.host.set_retry_at_for_test(at);
    }

    pub(crate) fn lose_connection_for_test(&mut self, reason: &str) {
        self.set_failure_for_test(Some(spaceport_endpoint::Failure::Transport {
            reason: Some(reason.into()),
            needs_user: false,
        }));
    }

    pub(crate) fn lose_herdr_for_test(&mut self, reason: &str) {
        self.set_failure_for_test(Some(spaceport_endpoint::Failure::Missing {
            reason: Some(reason.into()),
        }));
    }

    pub(crate) fn set_version_mismatch_for_test(&mut self, mismatch: Option<VersionMismatch>) {
        self.set_failure_for_test(
            mismatch.map(|mismatch| spaceport_endpoint::Failure::Version {
                mismatch,
                reason: None,
            }),
        );
    }

    fn set_failure_for_test(&mut self, why: Option<spaceport_endpoint::Failure>) {
        self.host
            .set_failure_for_test(self.connection.handle.is_some(), why);
    }

    pub(crate) fn attempts_for_test(&self) -> u32 {
        self.host.attempts_for_test()
    }

    pub(crate) fn set_attempts_for_test(&mut self, attempts: u32) {
        self.host.set_attempts_for_test(attempts);
    }

    pub(crate) fn detach_for_test(&mut self) {
        self.host.detach_for_test();
    }

    pub(crate) fn set_enabled_for_test(&mut self, enabled: bool) {
        self.host.set_enabled_for_test(enabled, self.transport());
    }

    #[cfg(unix)]
    pub(in crate::endpoint) fn online_since(&self) -> Option<Instant> {
        self.host.online_since()
    }

    #[cfg(unix)]
    pub(in crate::endpoint) fn clear_stability_for_test(&mut self) {
        self.host.clear_stability_for_test();
    }
}
