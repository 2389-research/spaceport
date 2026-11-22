use super::*;

#[test]
fn defaults_overrides_aliases_and_global_mute() {
    let settings = Settings::default();
    assert_eq!(settings.delay, 1);
    assert!(settings.sound.allows(None));
    assert!(settings.sound.allows(Some("claude")));
    assert!(!settings.sound.allows(Some("Droid")));
    let settings = Settings::parse("[ui.sound.agents]\ndroid = 'default'\nopen_code = 'off'\ngithub_copilot = 'off'\nagy = 'off'\n", Path::new("/config.toml")).unwrap();
    assert!(settings.sound.allows(Some("droid")));
    for alias in [
        "OpenCode",
        "opencode2",
        "open-code",
        "github-copilot",
        "copilot",
        "ghcs",
        "Antigravity",
        "C:\\bin\\opencode.exe",
    ] {
        assert!(!settings.sound.allows(Some(alias)), "{alias}");
    }
    assert!(settings.sound.allows(Some("unknown")));
    let settings = Settings::parse(
        "[ui.sound]\nenabled = false\n[ui.sound.agents]\ndroid = 'on'",
        Path::new("/config.toml"),
    )
    .unwrap();
    assert!(!settings.sound.allows(Some("droid")));
    assert!(!settings.sound.allows(None));
}

#[test]
fn paths_are_local_config_relative_with_per_sound_precedence() {
    let settings = Settings::parse("[ui.sound]\npath = 'all.mp3'\ndone_path = 'done.mp3'\n[ui.toast]\ndelay_seconds = 3600\n[unrelated]\nfield = 12", Path::new("/local/config.toml")).unwrap();
    assert_eq!(
        settings.path_for(Sound::Done),
        Some("/local/done.mp3".into())
    );
    assert_eq!(
        settings.path_for(Sound::Request),
        Some("/local/all.mp3".into())
    );
    assert_eq!(settings.delay, 3600);
    let settings = Settings::parse(
        "[ui.sound]\nrequest_path = '/absolute.mp3'",
        Path::new("/local/config.toml"),
    )
    .unwrap();
    assert_eq!(
        settings.path_for(Sound::Request),
        Some("/absolute.mp3".into())
    );
    assert_eq!(settings.path_for(Sound::Done), None);
}

#[test]
fn values_from_a_newer_herdr_fall_back_one_by_one() {
    // Herdr owns this file: a value this build cannot read keeps its own
    // default and every other setting still applies.
    for text in [
        "[ui.toast]\ndelay_seconds = 3601",
        "[ui.toast]\ndelay_seconds = -1",
        "[ui.toast]\ndelay_seconds = '2'",
        "[ui]\ntoast = 'later'",
    ] {
        assert_eq!(
            Settings::parse(text, Path::new("x")).unwrap().delay,
            1,
            "{text}"
        );
    }
    let settings = Settings::parse(
        "[ui.toast]\ndelay_seconds = 5\n[ui.sound]\nenabled = 'yes'\n\
             done_path = 'done.mp3'\nrequest_path = 7\nfuture = true\n\
             [ui.sound.agents]\nclaude = 'always'\ndroid = 'on'\ncodex = 'off'",
        Path::new("/local/config.toml"),
    )
    .unwrap();
    assert_eq!(settings.delay, 5);
    assert!(settings.sound.enabled);
    assert_eq!(
        settings.path_for(Sound::Done),
        Some("/local/done.mp3".into())
    );
    assert_eq!(settings.path_for(Sound::Request), None);
    // The unreadable agent setting keeps its default; the others apply.
    assert!(settings.sound.allows(Some("claude")));
    assert!(settings.sound.allows(Some("droid")));
    assert!(!settings.sound.allows(Some("codex")));
    let settings = Settings::parse("ui = 3", Path::new("x")).unwrap();
    assert!(settings.sound.enabled);
    assert_eq!(settings.delay, 1);
}

#[test]
fn bounded_reads_preserve_sources() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("config.toml");
    // A file Herdr has not written yet keeps every default.
    assert_eq!(Settings::load(&path).unwrap().delay, 1);
    std::fs::write(&path, "[ui.sound]\nenabled = false").unwrap();
    assert!(!Settings::load(&path).unwrap().sound.enabled);
    std::fs::write(&path, "[broken").unwrap();
    let error = Settings::load(&path).err().unwrap();
    assert!(
        matches!(&error, Error::Path { source, .. } if matches!(source.as_ref(), Error::Toml(_)))
    );
    assert!(std::error::Error::source(&error).is_some());
    std::fs::write(&path, vec![b' '; MAX_FILE_BYTES + 1]).unwrap();
    assert!(
        matches!(Settings::load(&path), Err(Error::Path { source, .. }) if matches!(*source, Error::ConfigSize))
    );
}
