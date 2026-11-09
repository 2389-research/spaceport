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
    fs::write(
        &source,
        format!("{constants}\nfn main() {{ assert_eq!(APP_VERSION, env!(\"SPACEPORT_APP_VERSION\")); assert_eq!(APP_ID, env!(\"SPACEPORT_APP_ID\")); assert!(!RELEASE_BUILD); let _ = (WINDOW_TITLE, LINUX_APP_ID, TAB_WIDTH, TAB_HEIGHT, BUILD_COMMIT); }}").replace("//!", "//"),
    ).unwrap();
    let compile = |release: bool, preview: bool| {
        let mut command = Command::new("rustc");
        command
            .arg("--edition=2024")
            .arg(&source)
            .arg("-o")
            .arg(temp.path().join("identity"))
            .env(
                "SPACEPORT_APP_ID",
                if preview {
                    "ai.2389.spaceport"
                } else {
                    "spaceport.development"
                },
            )
            .env(
                "SPACEPORT_APP_VERSION",
                if preview { "20261109.1" } else { "dev" },
            )
            .env_remove("HERDR_RELEASE_VERSION")
            .env_remove("SPACEPORT_MACOS_PREVIEW_VERSION");
        if release {
            command.env("HERDR_RELEASE_VERSION", "20990101.1");
        }
        if preview {
            command.env("SPACEPORT_MACOS_PREVIEW_VERSION", "20261109.1");
        }
        command.output().unwrap()
    };
    let development = compile(false, false);
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
    let preview = compile(false, true);
    assert!(
        preview.status.success(),
        "{}",
        String::from_utf8_lossy(&preview.stderr)
    );
    assert!(
        Command::new(temp.path().join("identity"))
            .status()
            .unwrap()
            .success()
    );
    let release = compile(true, false);
    assert!(!release.status.success());
    assert!(
        String::from_utf8_lossy(&release.stderr)
            .contains("Spaceport release identity is not configured")
    );
    let release_preview = compile(true, true);
    assert!(!release_preview.status.success());
    assert!(
        String::from_utf8_lossy(&release_preview.stderr)
            .contains("Spaceport release identity is not configured")
    );
}
