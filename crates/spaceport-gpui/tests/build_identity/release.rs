// ABOUTME: Compiles the app's release identity checks with explicit build inputs.
// ABOUTME: Covers development, signed preview and the disabled production channel.
use std::{fs, path::Path, process::Command};

#[test]
fn app_constants_accept_supported_builds_and_refuse_unconfigured_releases() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let app_name = manifest.join("../spaceport-client/src/app_name.rs");
    let constants = manifest.join("src/constants.rs");
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("identity.rs");
    let binary = temp
        .path()
        .join(format!("identity{}", std::env::consts::EXE_SUFFIX));
    fs::write(
        &source,
        format!(r#"
extern crate self as spaceport_client;
#[path = {app_name:?}] mod app_name;
pub use app_name::APP_NAME;
#[path = {constants:?}] mod constants;

fn main() {{
    assert_eq!(constants::APP_VERSION, env!("SPACEPORT_APP_VERSION"));
    assert_eq!(constants::APP_ID, env!("SPACEPORT_APP_ID"));
    assert_eq!(constants::WINDOW_TITLE, "Spaceport");
    assert_eq!(constants::HERDR_NAME, "Herdr");
    assert_eq!(Some(constants::CHANNEL), constants::Channel::parse(env!("SPACEPORT_BUILD_CHANNEL")));
    let _ = (constants::LINUX_APP_ID, constants::TAB_WIDTH, constants::TAB_HEIGHT, constants::BUILD_COMMIT);
}}
"#),
    ).unwrap();

    let compile = |channel: &str, release: bool| {
        let mut rustc = Command::new("rustc");
        rustc
            .args(["--edition=2024", "-o"])
            .arg(&binary)
            .arg(&source)
            .env("SPACEPORT_BUILD_CHANNEL", channel)
            .env_remove("HERDR_RELEASE_VERSION")
            .env_remove("SPACEPORT_MACOS_PREVIEW_VERSION")
            .env_remove("SPACEPORT_PRODUCTION_VERSION");
        if channel == "macos-preview" {
            rustc
                .env("SPACEPORT_APP_ID", "ai.2389.spaceport")
                .env("SPACEPORT_APP_VERSION", "20261109.1")
                .env("SPACEPORT_MACOS_PREVIEW_VERSION", "20261109.1");
        } else {
            rustc
                .env("SPACEPORT_APP_ID", "spaceport.development")
                .env("SPACEPORT_APP_VERSION", "dev");
        }
        if release {
            rustc.env("HERDR_RELEASE_VERSION", "20990101.1");
        }
        rustc.output().unwrap()
    };

    for channel in ["development", "macos-preview"] {
        let built = compile(channel, false);
        assert!(
            built.status.success(),
            "{channel}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        assert!(Command::new(&binary).status().unwrap().success());
    }
    for (channel, release, reason) in [
        (
            "development",
            true,
            "Spaceport release identity is not configured",
        ),
        (
            "macos-preview",
            true,
            "Spaceport release identity is not configured",
        ),
        (
            "release",
            false,
            "SPACEPORT_BUILD_CHANNEL is not a known build channel",
        ),
    ] {
        let refused = compile(channel, release);
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            !refused.status.success() && stderr.contains(reason),
            "{channel}: {stderr}"
        );
    }
}
