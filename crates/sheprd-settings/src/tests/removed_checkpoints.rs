// ABOUTME: Checks an old checkpoint setting is reported after worker removal.
// ABOUTME: Keeps a retained local setting usable in the same config text.
use super::*;

#[test]
fn old_checkpoint_setting_is_unknown_and_ports_still_load() -> anyhow::Result<()> {
    let config = Config::parse("agent_checkpoints = true\nshow_listening_ports = false")?;
    assert!(config.unknown_keys.contains(&"agent_checkpoints".into()));
    assert!(!config.show_listening_ports);
    Ok(())
}
