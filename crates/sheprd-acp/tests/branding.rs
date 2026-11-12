// ABOUTME: Checks the shipped helper's help without starting an agent or daemon.
// ABOUTME: Preserves its executable and environment contracts while naming Spaceport.
#![allow(clippy::unwrap_used)]
#[test]
fn helper_help_names_spaceport_and_keeps_compatibility_arguments() {
    let output = sheprd_client::new_command(env!("CARGO_BIN_EXE_sheprd-acp"))
        .arg("--help")
        .env_clear()
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(
        help.contains("shares the conversation with Spaceport"),
        "{help}"
    );
    assert!(help.contains("clients such as Spaceport drive the session"));
    assert!(help.contains("--no-attach    Do not listen for Spaceport"));
    assert!(help.contains("sheprd-acp connect ID"));
    assert!(help.contains("HERDR_ACP_ADAPTER=1"));
    assert!(!help.contains("Sheprd"));
    assert!(output.stderr.is_empty());
}
