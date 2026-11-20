// ABOUTME: Checks runtime configuration path selection independently of client views.
// ABOUTME: Preserves explicit Herdr overrides and native configuration roots.
use super::*;

#[test]
fn runtime_config_uses_herdr_override_or_its_release_root() {
    let root = env::temp_dir().join("fixture-config-root");
    let explicit = root.join("runtime-override.toml");
    for override_path in [
        None,
        Some(OsString::new()),
        Some(explicit.clone().into_os_string()),
    ] {
        let path = herdr_config_path_with(|name| match name {
            "HERDR_CONFIG_PATH" => override_path.clone(),
            "XDG_CONFIG_HOME" => Some(root.clone().into_os_string()),
            _ => None,
        });
        assert_eq!(
            path,
            override_path
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("herdr/config.toml"))
        );
        assert_ne!(path, root.join("spaceport/view.toml"));
    }
}
