//! Keeps `herdr-client`'s unverified-host list in step with the settings: now,
//! from what the app loaded at startup, and after every edit of the settings
//! files. The list itself is `sheprd-settings`' `[ssh]` table.
use crate::config::SshConfig;

pub(crate) fn install(startup: &SshConfig, cx: &mut gpui::App) {
    apply(cx, startup.clone());
    super::watch::follow(cx, "ssh", |config| config.ssh, apply).detach();
}

fn apply(_: &mut gpui::App, ssh: SshConfig) {
    sheprd_client::set_unverified_hosts(ssh.skip_host_key_verification);
}
