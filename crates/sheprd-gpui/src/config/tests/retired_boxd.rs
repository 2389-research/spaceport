// ABOUTME: Checks old Boxd tables cannot revive a removed cloud provider.
// ABOUTME: Keeps unsupported setup explicit and leaves user settings untouched.
use super::*;

#[test]
fn retired_boxd_account_is_unknown_without_activating_provider() -> anyhow::Result<()> {
    let config = Config::parse("[boxd]\napi_url = 'https://boxd.sh:9443'\n")?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("boxd"))
    );
    Ok(())
}

#[test]
fn retired_boxd_setup_refuses_load_without_rewriting_user_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.toml");
    let original = b"[cloud.boxd]\nsetup = 'echo old'\n";
    fs::write(&path, original)?;
    assert!(Config::load_startup_path(&path, &dir.path().join("absent-daemon.toml")).is_err());
    assert_eq!(fs::read(path)?, original);
    Ok(())
}
