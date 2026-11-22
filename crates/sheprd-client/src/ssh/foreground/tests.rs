// ABOUTME: Checks the ForkAfterAuthentication probe against the installed OpenSSH parser.
// ABOUTME: Uses an empty private configuration and never connects to a host.
use super::*;

/// The installed `ssh`, reading only an empty private configuration.
fn private_ssh(root: &std::path::Path) -> Command {
    let config = root.join("config");
    std::fs::write(&config, "").unwrap();
    let mut ssh = Command::new(crate::environment::command("ssh").get_program());
    ssh.env_clear().env("HOME", root);
    #[cfg(windows)]
    ssh
        // Windows OpenSSH requires ProgramData before initializing its stderr.
        .env("ProgramData", root)
        .env("SystemRoot", std::env::var_os("SystemRoot").unwrap());
    ssh.arg("-F").arg(config);
    ssh
}

#[test]
fn a_current_client_accepts_the_foreground_option() {
    let root = tempfile::tempdir().unwrap();
    assert!(accepts(private_ssh(root.path()), OPTION));
}

#[test]
fn a_client_that_does_not_know_an_option_is_not_given_it() {
    let root = tempfile::tempdir().unwrap();
    assert!(!accepts(
        private_ssh(root.path()),
        "SpaceportOptionNoClientKnows=no"
    ));
}
