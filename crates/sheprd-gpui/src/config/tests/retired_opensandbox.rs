// ABOUTME: Checks old OpenSandbox tables cannot revive a removed cloud provider.
// ABOUTME: Keeps unsupported setup explicit and leaves user settings untouched.
use super::*;

#[test]
fn retired_opensandbox_account_is_unknown_without_activating_provider() -> anyhow::Result<()> {
    let config = Config::parse("[opensandbox]\napi_url = 'https://sandbox.example.com'\n")?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("opensandbox"))
    );
    Ok(())
}

#[test]
fn retired_opensandbox_setup_is_ignored_without_rewriting_user_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.toml");
    let original = b"[cloud.opensandbox]\nsetup = 'echo old'\n";
    fs::write(&path, original)?;
    let config = Config::load_startup_path(&path, &dir.path().join("absent-daemon.toml"))?;
    assert_eq!(config.unknown_keys, ["cloud"]);
    assert_eq!(fs::read(path)?, original);
    Ok(())
}
