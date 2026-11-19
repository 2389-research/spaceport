// ABOUTME: Checks old Azure cloud setup cannot revive removed provisioning.
// ABOUTME: Keeps unsupported setup explicit and leaves user settings untouched.
use super::*;

#[test]
fn retired_azure_cloud_account_is_unknown_without_activating_provider() -> anyhow::Result<()> {
    let config = Config::parse("[azure]\nsubscription_id = 'subscription-fixture'\n")?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("azure"))
    );
    Ok(())
}

#[test]
fn retired_azure_setup_is_ignored_without_rewriting_user_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.toml");
    let original = b"[cloud.azure]\nsetup = 'echo old'\n";
    fs::write(&path, original)?;
    let config = Config::load_startup_path(&path, &dir.path().join("absent-daemon.toml"))?;
    assert_eq!(config.unknown_keys, ["cloud"]);
    assert_eq!(fs::read(path)?, original);
    Ok(())
}
