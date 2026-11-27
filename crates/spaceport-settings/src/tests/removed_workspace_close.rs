use super::*;

#[test]
fn old_workspace_close_preference_is_reported_as_unknown() -> anyhow::Result<()> {
    let config = Config::parse("confirm_close_workspace = false")?;
    assert_eq!(config.unknown_keys, ["confirm_close_workspace"]);
    Ok(())
}
