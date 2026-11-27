// ABOUTME: Checks old Railway setup cannot revive a removed cloud provider.
// ABOUTME: Keeps unsupported setup explicit and leaves user settings untouched.
use super::*;

#[test]
fn retired_railway_setup_is_ignored_without_rewriting_user_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("config.toml");
    let original = b"[cloud.railway]\nsetup = 'echo old'\n";
    fs::write(&path, original)?;
    let config = Config::load_startup_path(&path, &dir.path().join("absent-daemon.toml"))?;
    assert_eq!(config.unknown_keys, ["cloud"]);
    assert_eq!(fs::read(path)?, original);
    Ok(())
}
