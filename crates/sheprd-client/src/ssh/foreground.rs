// ABOUTME: Keeps owned SSH children in the foreground where the client can be told to.
// ABOUTME: Checks once whether this ssh knows ForkAfterAuthentication before passing it.
use std::{process::Command, sync::OnceLock, time::Duration};

const OPTION: &str = "ForkAfterAuthentication=no";
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// `-o ForkAfterAuthentication=no`, so a user's `ForkAfterAuthentication yes`
/// cannot send a child this crate owns into the background, or nothing when
/// this `ssh` predates the option (OpenSSH 8.7). An older client refuses to
/// start with an option it does not know, and its config cannot set this one
/// either, so leaving it out there changes nothing. Checked once per process
/// with `ssh -G`, which reads configuration and never connects; that blocks
/// the first time, so call this off the UI thread.
pub(crate) fn options() -> &'static [&'static str] {
    static KNOWN: OnceLock<bool> = OnceLock::new();
    if *KNOWN.get_or_init(|| accepts(crate::environment::command("ssh"), OPTION)) {
        &["-o", OPTION]
    } else {
        &[]
    }
}

/// Whether `ssh` parses `option`, without connecting anywhere.
fn accepts(mut ssh: Command, option: &str) -> bool {
    ssh.args(["-G", "-o", option, "--", "localhost"]);
    super::run(&mut ssh, CHECK_TIMEOUT, || false).is_ok_and(|(status, _)| status.success())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
