// ABOUTME: Rejects retired updater settings at the configuration boundary.
// ABOUTME: Covers table, dotted and inline TOML forms without silently changing channels.
use super::*;

#[test]
fn removed_updater_settings_are_rejected() {
    for source in [
        "[updates]\nchannel = 'beta'",
        "updates = {channel = 'stable'}",
        "updates.channel = 'beta'",
    ] {
        assert!(matches!(
            Config::parse(source),
            Err(crate::Error::Settings(Error::RemovedUpdates))
        ));
        assert!(matches!(
            Config::parse_layers([DEFAULT_CONFIG, source], &Daemon::default()),
            Err(crate::Error::Settings(Error::RemovedUpdates))
        ));
    }
}
