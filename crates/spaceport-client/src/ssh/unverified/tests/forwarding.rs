// ABOUTME: Checks unverified host forwarding against the installed OpenSSH parser.
// ABOUTME: Uses a private configuration without connecting to a host or reading keys.

use super::*;
use std::process::Command;

fn effective_options(command: &Command) -> String {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    std::fs::write(
        &config,
        "Host *\n    ForwardAgent yes\n    ForwardX11 yes\n    CanonicalizeHostname no\n",
    )
    .unwrap();
    let mut probe = crate::new_command(command.get_program());
    probe.env_clear().env("HOME", root.path());
    #[cfg(windows)]
    probe
        // Windows OpenSSH initializes stderr through these system paths.
        // Explicit -F excludes the default system and user configuration.
        .env("ProgramData", root.path())
        .env("SystemRoot", std::env::var_os("SystemRoot").unwrap());
    probe
        .args(["-G", "-F"])
        .arg(config)
        .args(command.get_args())
        .stderr(std::process::Stdio::piped());
    let (status, output, stderr) =
        crate::ssh::run_with_stderr(&mut probe, crate::ssh::RESOLVE_TIMEOUT, || false).unwrap();
    assert!(
        status.success(),
        "ssh -G failed for {:?}: {status}; {}",
        command.get_program(),
        String::from_utf8_lossy(&stderr)
    );
    String::from_utf8(output).unwrap()
}

#[test]
fn skipped_connections_disable_configured_agent_and_x11_forwarding() {
    let commands = crate::tests::while_unverified("forwarding-skipped.invalid", || {
        [
            crate::ssh::command("forwarding-skipped.invalid", "true"),
            crate::ssh::forward_command("forwarding-skipped.invalid", 59289, None, 443).unwrap(),
        ]
    });
    for command in commands {
        let options = effective_options(&command);
        assert!(options.lines().any(|line| line == "forwardagent no"));
        assert!(options.lines().any(|line| line == "forwardx11 no"));
    }
}

#[test]
fn verified_connections_keep_configured_agent_forwarding() {
    let command = crate::ssh::command("forwarding-verified.invalid", "true");
    let options = effective_options(&command);
    assert!(options.lines().any(|line| line == "forwardagent yes"));
    assert!(options.lines().any(|line| line == "forwardx11 no"));
}

#[test]
fn skipped_host_policy_disables_configured_forwarding() {
    let mut command = crate::new_command("ssh");
    command
        .args(HostKeys::Skip.options())
        .args(["--", "forwarding-policy.invalid"]);
    let options = effective_options(&command);
    assert!(options.lines().any(|line| line == "forwardagent no"));
    assert!(options.lines().any(|line| line == "forwardx11 no"));
}
