// ABOUTME: Exercises remote helper discovery in current Spaceport bundle locations.
// ABOUTME: Rebases absolute app roots so tests cannot reach personal installations.
use super::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn spaceport_bundle_discovery_finds_user_and_system_installations() {
    for user_install in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let system = root.path().join("system-applications");
        let empty_path = root.path().join("bin");
        std::fs::create_dir_all(&empty_path).unwrap();
        let apps = if user_install {
            home.join("Applications")
        } else {
            system.clone()
        };
        let helper = apps.join("Spaceport.app/Contents/MacOS/sheprd-acp");
        std::fs::create_dir_all(helper.parent().unwrap()).unwrap();
        std::fs::write(&helper, "#!/bin/sh\nprintf '%s\\n' \"found:$1\"\n").unwrap();
        std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755)).unwrap();
        let script = remote_program_script("agents")
            .replace("\"/Applications/", &format!("\"{}/", system.display()));
        assert!(!script.contains("\"/Applications/"));
        let output = sheprd_client::new_command("/bin/sh")
            .args(["-c", &script])
            .env_clear()
            .env("HOME", &home)
            .env("PATH", &empty_path)
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "user_install={user_install}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"found:agents\n");
    }
}
