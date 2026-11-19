// ABOUTME: Rejects excluded provider sign-in settings at the app's config boundary.
// ABOUTME: Covers direct, inline and merged configuration without reading credentials.
use super::*;

#[test]
fn provider_sign_in_settings_are_rejected() {
    for source in [
        "[usage]\nbrowser_cookies = false",
        "[usage.providers.openai]\napi_key = 'fixture'",
        "[usage.providers]",
        "usage = { providers = {} }",
    ] {
        assert!(Config::parse(source).is_err(), "accepted {source}");
        assert!(
            Config::parse_layers(["[usage]\nshow = true", source], &Daemon::default()).is_err(),
            "accepted layered {source}"
        );
    }
}
