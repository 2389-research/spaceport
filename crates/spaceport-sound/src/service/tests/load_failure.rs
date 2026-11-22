//! A settings load can fail on a file that holds the user's own text: the
//! worker's log line keeps the failure's kind and none of its message.
use super::*;
use crate::Error;

/// `Settings::load`'s error for a config whose broken line holds user text.
fn load_failure(dir: &Path) -> Error {
    let config = dir.join("config.toml");
    std::fs::write(
        &config,
        "[ui.sound]\npath = 'ding.mp3'\nagent secret note\n",
    )
    .unwrap();
    let Err(error) = Settings::load(&config) else {
        panic!("a broken config must fail to load");
    };
    error
}

#[test]
fn a_failed_load_logs_its_kind_and_not_the_line_or_path_it_names() {
    let dir = tempfile::tempdir().unwrap();
    let error = load_failure(dir.path());
    // The message the worker used to log: the file's path, then the line that
    // broke, quoted from the user's config by the TOML parser.
    let message = error.to_string();
    assert!(message.contains("config.toml"), "{message}");
    assert!(message.contains("agent secret note"), "{message}");
    // What it logs instead.
    assert_eq!(error.kind(), "toml");
    for leaked in ["config.toml", "agent secret note", "ding.mp3"] {
        assert!(!error.kind().contains(leaked), "{leaked} in log");
    }
}
