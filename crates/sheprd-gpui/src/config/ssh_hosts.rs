//! `[ssh]`: SSH hosts the user chose to connect to without verifying their
//! host keys, by the target saved with the device. The app pushes the list to
//! `herdr-client`, whose `ssh` children then accept any key those hosts offer.
use crate::Result;
use serde::Deserialize;
use sheprd_client::MAX_UNVERIFIED_HOSTS;
use std::collections::BTreeSet;

mod error;
pub use error::Error;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SshConfig {
    /// Targets whose host keys are never checked. Anyone able to intercept
    /// the connection to one can impersonate it.
    pub skip_host_key_verification: BTreeSet<String>,
}

impl SshConfig {
    pub(super) fn check(&self) -> Result<()> {
        if self.skip_host_key_verification.len() > MAX_UNVERIFIED_HOSTS {
            return Err(Error::TooMany(MAX_UNVERIFIED_HOSTS).into());
        }
        if let Some(target) = self
            .skip_host_key_verification
            .iter()
            .find(|target| !valid_target(target))
        {
            return Err(Error::InvalidTarget(target.clone()).into());
        }
        Ok(())
    }
}

/// An SSH target that cannot read as an option or as several words.
fn valid_target(target: &str) -> bool {
    !target.is_empty()
        && target.len() <= 1024
        && !target.starts_with('-')
        && !target.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// Keeps `herdr-client`'s list in step with the settings: now, from what the
/// app loaded at startup, and after every edit of the settings files.
pub(crate) fn install(startup: &SshConfig, cx: &mut gpui::App) {
    apply(cx, startup.clone());
    super::watch::follow(cx, "ssh", |config| config.ssh, apply).detach();
}

fn apply(_: &mut gpui::App, ssh: SshConfig) {
    sheprd_client::set_unverified_hosts(ssh.skip_host_key_verification);
}
