// ABOUTME: Saves settings through the executable against isolated real files.
// ABOUTME: Checks existing permissions and dotfile symlinks survive CLI edits.
use super::*;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};

#[test]
fn config_set_preserves_override_modes_and_symlink_targets() {
    warm_up();
    let root = tempfile::Builder::new()
        .prefix("sp-mode")
        .tempdir_in("/tmp")
        .unwrap();
    let config = root.path().join("config");
    let app = config.join("spaceport");
    fs::create_dir_all(&app).unwrap();
    let local = app.join("config.local.toml");
    for linked in [false, true] {
        let target = if linked {
            fs::remove_file(&local).unwrap();
            let target = root.path().join("dotfile.toml");
            std::os::unix::fs::symlink(&target, &local).unwrap();
            target
        } else {
            local.clone()
        };
        fs::write(&target, "focus_follows_mouse = false\n").unwrap();
        let file = fs::File::open(&target).unwrap();
        if let Some(group) = rustix::process::getgroups()
            .unwrap()
            .into_iter()
            .find(|group| group.as_raw() != file.metadata().unwrap().gid())
        {
            rustix::fs::fchown(&file, None, Some(group)).unwrap();
        }
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        let before = fs::metadata(&target).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_spaceport"))
            .env_clear()
            .env("HOME", root.path())
            .env("XDG_CONFIG_HOME", &config)
            .env("XDG_STATE_HOME", root.path().join("state"))
            .env("PATH", "/usr/bin:/bin")
            .args(["config", "set", "focus_follows_mouse", "true"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = wait(child, &["config", "set"], COMMAND_LIMIT);
        assert!(output.status.success(), "{output:?}");
        let text = fs::read_to_string(&target).unwrap();
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert_eq!(value["focus_follows_mouse"].as_bool(), Some(true));
        let after = fs::metadata(&target).unwrap();
        assert_eq!(after.permissions().mode() & 0o777, 0o640);
        assert_eq!(after.uid(), before.uid());
        assert_eq!(after.gid(), before.gid());
        if linked {
            assert_eq!(fs::read_link(&local).unwrap(), target);
        }
    }
}
