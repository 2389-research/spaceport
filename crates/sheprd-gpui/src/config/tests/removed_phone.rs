use super::*;

#[test]
fn stale_phone_settings_are_inert_and_reported_as_unknown() -> anyhow::Result<()> {
    let config = Config::parse("[remote]\nenabled = true\nport = 7443\n")?;
    assert_eq!(config.unknown_keys, ["remote"]);
    Ok(())
}
