// ABOUTME: Keep a failed native GUI's decisive handoff state in its test failure.
// ABOUTME: Exercise the failure reporter without launching a desktop or a daemon.
use super::*;

pub(super) fn assert_startup_capture(root: &Path) {
    let path = root.join("state/spaceport/logs/spaceport.jsonl");
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => panic!(
            "cannot read private diagnostic log {}: {error}",
            path.display()
        ),
    };
    let namespace = Path::new(env!("CARGO_BIN_EXE_spaceport"))
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let records: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        records.iter().any(|record| record["namespace"] == namespace
            && record["message"] == "GPUI client starting"),
        "expected the real GUI startup event in private diagnostic log {}",
        path.display()
    );
}

#[test]
fn failure_log_tail_preserves_bounds_and_unicode() {
    let mut log = (0..50)
        .map(|n| format!("old-line-{n}\n"))
        .collect::<String>();
    log.push_str(&"界".repeat(601));
    let bounded = fixture_log_tail(&log);
    assert_eq!(bounded.lines().count(), 40);
    assert!(!bounded.contains("old-line-0\n"));
    assert!(bounded.lines().all(|line| line.chars().count() <= 600));
    assert_eq!(bounded.lines().last(), Some("界".repeat(600).as_str()));
}

#[test]
#[should_panic(expected = "shell_ready=false")]
fn unsuccessful_gui_reports_its_owned_log_before_failing() {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    #[cfg(unix)]
    let status = ExitStatus::from_raw(1 << 8);
    #[cfg(windows)]
    let status = ExitStatus::from_raw(1);
    assert_gui_success(status, "GUI step=8 tab_match=true shell_ready=false");
}
