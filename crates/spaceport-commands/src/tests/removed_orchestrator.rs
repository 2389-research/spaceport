use super::*;

#[test]
fn direct_orchestrator_command_is_absent() {
    assert!(COMMANDS.iter().all(|info| info.name != "open_orchestrator"));
}
