// ABOUTME: Checks old Sprites settings cannot revive a removed provider.
// ABOUTME: Keeps unsupported setup explicit and leaves user settings untouched.
use super::*;

#[test]
fn retired_sprites_account_is_unknown_without_activating_provider() -> anyhow::Result<()> {
    let config = Config::parse("[sprites]\napi_url = 'https://api.sprites.dev'\n")?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("sprites"))
    );
    Ok(())
}

#[test]
fn retired_sprites_setup_refuses_load_without_rewriting_user_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.toml");
    let original = b"[cloud.sprites]\nsetup = 'echo old'\n";
    fs::write(&path, original)?;
    assert!(Config::load_startup_path(&path, &dir.path().join("absent-daemon.toml")).is_err());
    assert_eq!(fs::read(path)?, original);
    Ok(())
}
