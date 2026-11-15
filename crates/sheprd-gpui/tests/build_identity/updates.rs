// ABOUTME: Builds the real build script in an isolated dependency-free host.
// ABOUTME: Verifies updater framework search paths on binaries and examples.
use super::*;

fn fixture() -> Sandbox {
    let root = Sandbox::new();
    let path = &root.0;
    fs::create_dir_all(path.join("src")).unwrap();
    fs::create_dir_all(path.join("examples")).unwrap();
    fs::write(path.join("Cargo.toml"), "[package]\nname = \"updater-linkage\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[features]\nmacos-updater = []\n[workspace]\n").unwrap();
    for source in ["src/main.rs", "examples/host.rs"] {
        fs::write(path.join(source), "fn main() {}\n").unwrap();
    }
    for (name, source) in [
        ("build.rs", include_str!("../../build.rs")),
        ("build_identity.rs", include_str!("../../build_identity.rs")),
        (
            "preview_identity.rs",
            include_str!("../../preview_identity.rs"),
        ),
        (
            "release_version.rs",
            include_str!("../../release_version.rs"),
        ),
        (
            "development-app-id",
            include_str!("../../development-app-id"),
        ),
        (
            "macos-preview-app-id",
            include_str!("../../macos-preview-app-id"),
        ),
    ] {
        fs::write(path.join(name), source).unwrap();
    }
    // Locale generation and Windows resources are unrelated to this linker fixture.
    fs::write(path.join("locales.rs"), "#[derive(Debug)] pub enum Error {}\nimpl std::fmt::Display for Error { fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match *self {} } }\nimpl std::error::Error for Error {}\npub fn generate(_: &std::path::Path, _: &std::path::Path) -> Result<(), Error> { Ok(()) }\n").unwrap();
    fs::write(path.join("windows_resource.rs"), "pub fn embed(_: &std::path::Path, _: bool) -> std::io::Result<()> { let _ = crate::release_version::numeric; Ok(()) }\n").unwrap();
    root
}

fn build(root: &Sandbox, feature: bool) {
    let mut command = Command::new(env!("CARGO"));
    command
        .current_dir(&root.0)
        .args([
            "build",
            "--offline",
            "--examples",
            "--bin",
            "updater-linkage",
        ])
        .env("CARGO_TARGET_DIR", root.0.join("target"))
        .env("HERDR_BUILD_PR_NUMBER", "")
        .env_remove("SPACEPORT_MACOS_PREVIEW_VERSION")
        .env_remove("CARGO_FEATURE_MACOS_UPDATER");
    if feature {
        command.args(["--features", "macos-updater"]);
    }
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
}

/// The compiled build script of a fixture that `build` has built.
fn build_script(root: &Sandbox) -> PathBuf {
    let name = format!("build-script-build{}", std::env::consts::EXE_SUFFIX);
    fs::read_dir(root.0.join("target/debug/build"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join(&name))
        .find(|path| path.is_file())
        .unwrap()
}

#[test]
fn build_script_emits_the_channel_of_each_identity() {
    let root = fixture();
    build(&root, false);
    let script = build_script(&root);
    let run = |target: &str, preview: Option<&str>| {
        let mut command = Command::new(&script);
        command
            .current_dir(&root.0)
            .env("CARGO_MANIFEST_DIR", &root.0)
            .env("OUT_DIR", &root.0)
            .env("CARGO_CFG_TARGET_OS", target)
            .env("HERDR_BUILD_PR_NUMBER", "")
            .env_remove("CARGO_FEATURE_MACOS_UPDATER")
            .env_remove("SPACEPORT_MACOS_PREVIEW_VERSION");
        if let Some(preview) = preview {
            command.env("SPACEPORT_MACOS_PREVIEW_VERSION", preview);
        }
        command.output().unwrap()
    };
    let development = include_str!("../../development-app-id").trim();
    let preview = include_str!("../../macos-preview-app-id").trim();
    for (target, version, channel, id, shown) in [
        ("macos", None, "development", development, "dev"),
        ("linux", None, "development", development, "dev"),
        ("windows", None, "development", development, "dev"),
        (
            "macos",
            Some("20261109.1"),
            "macos-preview",
            preview,
            "20261109.1",
        ),
    ] {
        let output = run(target, version);
        assert!(output.status.success(), "{target} {version:?}: {output:?}");
        let directives = String::from_utf8(output.stdout).unwrap();
        let emitted = |name: &str| {
            let prefix = format!("cargo:rustc-env={name}=");
            directives
                .lines()
                .filter_map(|line| line.strip_prefix(&prefix))
                .collect::<Vec<_>>()
        };
        assert_eq!(emitted("SPACEPORT_BUILD_CHANNEL"), [channel], "{target}");
        assert_eq!(emitted("SPACEPORT_APP_ID"), [id], "{target}");
        assert_eq!(emitted("SPACEPORT_APP_VERSION"), [shown], "{target}");
    }
    // A preview version is refused off macOS, so no channel is emitted for it.
    let refused = run("linux", Some("20261109.1"));
    assert!(!refused.status.success());
    assert!(!String::from_utf8_lossy(&refused.stdout).contains("SPACEPORT_BUILD_CHANNEL"));
}

#[cfg(target_os = "macos")]
#[test]
fn only_enabled_macos_hosts_receive_the_bundled_framework_search_path() {
    let root = fixture();
    for feature in [false, true, false] {
        build(&root, feature);
        for name in ["updater-linkage", "examples/host"] {
            let output = Command::new("otool")
                .arg("-l")
                .arg(root.0.join("target/debug").join(name))
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            let commands = String::from_utf8(output.stdout).unwrap();
            assert_eq!(
                commands.contains("path @executable_path/../Frameworks (offset"),
                feature,
                "feature={feature}, {name}: {commands}"
            );
        }
    }
}

#[test]
fn non_macos_target_does_not_receive_the_updater_search_path() {
    let root = fixture();
    build(&root, true);
    let script = build_script(&root);
    for target in ["linux", "windows"] {
        let output = Command::new(&script)
            .current_dir(&root.0)
            .env("CARGO_MANIFEST_DIR", &root.0)
            .env("OUT_DIR", &root.0)
            .env("CARGO_CFG_TARGET_OS", target)
            .env("CARGO_FEATURE_MACOS_UPDATER", "1")
            .env("HERDR_BUILD_PR_NUMBER", "")
            .env_remove("SPACEPORT_MACOS_PREVIEW_VERSION")
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let directives = String::from_utf8(output.stdout).unwrap();
        assert!(
            !directives.contains("rustc-link-arg"),
            "{target}: {directives}"
        );
    }
}
