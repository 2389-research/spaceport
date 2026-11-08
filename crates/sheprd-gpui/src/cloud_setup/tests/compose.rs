use super::*;

#[test]
fn nothing_picked_composes_no_script() {
    assert_eq!(compose([], None).unwrap(), None);
    assert_eq!(compose([], Some("  \n ")).unwrap(), None);
}

#[test]
fn agents_install_once_in_display_order_before_the_users_lines() {
    let script = compose(
        [Agent::OpenCode, Agent::Claude, Agent::OpenCode],
        Some("\ngit config --global pull.rebase true\n"),
    )
    .unwrap()
    .unwrap();
    let claude = script.find("Installing Claude Code").unwrap();
    let opencode = script.find("Installing OpenCode").unwrap();
    let custom = script.find("git config").unwrap();
    assert!(claude < opencode && opencode < custom);
    assert_eq!(script.matches("Installing OpenCode").count(), 1);
    assert!(script.ends_with("pull.rebase true\n"));
}

#[test]
fn a_custom_script_alone_runs_without_the_agents_note() {
    let script = compose([], Some("echo hi")).unwrap().unwrap();
    assert_eq!(script, "echo hi\n");
}

#[test]
fn a_nul_or_oversized_script_is_refused() {
    assert!(matches!(compose([], Some("a\0b")), Err(Error::ScriptNul)));
    let long = "x".repeat(MAX_SCRIPT);
    assert!(matches!(
        compose([Agent::Claude], Some(&long)),
        Err(Error::ScriptTooLong { limit: MAX_SCRIPT })
    ));
}
