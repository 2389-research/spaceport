// ABOUTME: Guards the custom updater deletion and retained release identity gate.
// ABOUTME: Compiles the actual app constants in isolation without launching the app.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]
use std::{fs, path::Path, process::Command};

#[test]
fn custom_updater_has_no_source_or_dependency() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(!root.join("crates/sheprd-updater").exists());
    for file in ["Cargo.toml", "Cargo.lock", "crates/sheprd-gpui/Cargo.toml"] {
        assert!(
            !fs::read_to_string(root.join(file))
                .unwrap()
                .contains("sheprd-updater"),
            "{file}"
        );
    }
}

#[test]
fn app_constants_retain_development_identity_and_release_refusal() {
    let constants =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/constants.rs")).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("identity.rs");
    // The credential policy is a separate dependency; this fixture supplies only
    // its existing boolean while compiling every actual app constant unchanged.
    fs::write(&source, format!("mod sheprd_credentials {{ pub const RELEASE_BUILD: bool = false; }}\n{constants}\nfn main() {{ assert_eq!(APP_VERSION, \"dev\"); assert_eq!(APP_ID, \"spaceport.development\"); let _ = (WINDOW_TITLE, LINUX_APP_ID, TAB_WIDTH, TAB_HEIGHT, RELEASE_BUILD, BUILD_COMMIT); }}").replace("//!", "//")).unwrap();
    let compile = |release: bool| {
        let mut command = Command::new("rustc");
        command
            .arg("--edition=2024")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("identity"));
        command.env_remove("HERDR_RELEASE_VERSION");
        if release {
            command.env("HERDR_RELEASE_VERSION", "20990101.1");
        }
        command.output().unwrap()
    };
    let development = compile(false);
    assert!(
        development.status.success(),
        "{}",
        String::from_utf8_lossy(&development.stderr)
    );
    assert!(
        Command::new(temp.path().join("identity"))
            .status()
            .unwrap()
            .success()
    );
    let release = compile(true);
    assert!(!release.status.success());
    assert!(
        String::from_utf8_lossy(&release.stderr)
            .contains("Spaceport release identity is not configured")
    );
}
