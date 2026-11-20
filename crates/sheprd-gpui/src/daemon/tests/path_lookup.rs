// ABOUTME: Keeps automatic Herdr discovery out of the current checkout.
// ABOUTME: Runs PATH probes in private children without changing the test process.
use super::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn daemon_lookup_ignores_relative_path_entries() {
    const PROBE: &str = "SPACEPORT_HERDR_PATH_PROBE";
    if let Some(expected) = env::var_os(PROBE) {
        assert_eq!(executable().unwrap(), PathBuf::from(expected));
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let trusted = root.path().join("trusted");
    let untrusted = root.path().join("untrusted");
    std::fs::create_dir(&trusted).unwrap();
    std::fs::create_dir(&untrusted).unwrap();
    for dir in [root.path(), &trusted, &untrusted] {
        let program = dir.join(NAME);
        std::fs::write(&program, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(program, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    for relative in [".", "", "untrusted"] {
        let output = Command::new(env::current_exe().unwrap())
            .args([
                "--exact",
                "daemon::tests::path_lookup::daemon_lookup_ignores_relative_path_entries",
                "--nocapture",
            ])
            .env_clear()
            .env(PROBE, trusted.join(NAME))
            .env("HOME", root.path())
            .env("TMPDIR", root.path())
            .env(
                "PATH",
                env::join_paths([Path::new(relative), &trusted]).unwrap(),
            )
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{relative:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn missing_installations_return_an_error_without_a_bare_fallback() {
    let root = tempfile::tempdir().unwrap();
    let error = installed_executable([PathBuf::from("herdr"), root.path().join("not-installed")])
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(is_missing_installation(&error));
}

#[test]
fn a_lookup_error_is_reported_only_after_a_connection_attempt() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("fixture.sock");
    let _listener = UnixListener::bind(&path).unwrap();
    let stream = connect_or_start(
        &path,
        &AtomicBool::new(false),
        Duration::ZERO,
        || installed_executable([]).map(Command::new),
        true,
    )
    .unwrap();
    drop(stream);
    let error = connect_or_start(
        &root.path().join("missing.sock"),
        &AtomicBool::new(false),
        Duration::ZERO,
        || installed_executable([]).map(Command::new),
        true,
    )
    .unwrap_err();
    assert!(is_missing_installation(&error));
}
