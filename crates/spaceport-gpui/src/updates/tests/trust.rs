// ABOUTME: Exercises the compiled Sparkle trust inputs and their https rule.
// ABOUTME: Pins each signed channel's feed so a channel cannot follow another's.
#![allow(clippy::unwrap_used)]
use super::*;

const PREVIEW: &str =
    "https://github.com/2389-research/spaceport/releases/download/macos-preview-feed/appcast.xml";
const STABLE: &str =
    "https://github.com/2389-research/spaceport/releases/download/macos-stable-feed/appcast.xml";
const KEY: &str = "WoB5mqAl0CXMo2e5dKUGl8Ayna8gzQbIkiXeM+YizjI=";

fn config(preview: &str, production: &str) -> String {
    format!(
        r#"{{"sdk_version":"2.10.1","sdk_sha256":"ignored","public_key":"{KEY}","feeds":{{"macos-preview":{{"url":"{preview}","release":"macos-preview-feed"}},"production":{{"url":"{production}","release":"macos-stable-feed"}}}}}}"#
    )
}

#[test]
fn each_signed_channel_checks_its_own_appcast() {
    let trust = Trust::parse(&config(PREVIEW, STABLE)).unwrap();
    assert_eq!(trust.feed(Channel::MacosPreview), Some(PREVIEW));
    assert_eq!(trust.feed(Channel::Production), Some(STABLE));
    assert_eq!(trust.public_key, KEY);
}

#[test]
fn development_never_has_a_feed() {
    let trust = Trust::parse(&config(PREVIEW, STABLE)).unwrap();
    assert_eq!(trust.feed(Channel::Development), None);
}

#[test]
fn a_channel_the_configuration_omits_cannot_update() {
    let config = r#"{"public_key":"KEY","feeds":{"macos-preview":{"url":"https://example.test/preview/appcast.xml"}}}"#;
    let trust = Trust::parse(config).unwrap();
    assert!(trust.feed(Channel::MacosPreview).is_some());
    assert_eq!(trust.feed(Channel::Production), None);
}

#[test]
fn the_compiled_configuration_names_the_preview_and_stable_feeds() {
    let trust = Trust::compiled().unwrap();
    assert_eq!(trust.feed(Channel::MacosPreview), Some(PREVIEW));
    assert_eq!(trust.feed(Channel::Production), Some(STABLE));
    assert_eq!(trust.public_key.len(), KEY.len());
}

#[test]
fn a_feed_url_that_is_not_https_fails_closed() {
    for url in [
        "http://example.test/appcast.xml",
        "//example.test/appcast.xml",
        "/appcast.xml",
        "https://",
        "https:///appcast.xml",
        "https://user@appcast.test/appcast.xml",
        "https://example.test",
        "",
    ] {
        assert!(!is_https_url(url), "accepted {url:?}");
        assert!(
            Trust::parse(&config(url, STABLE)).is_err(),
            "parsed {url:?}"
        );
    }
}

#[test]
fn an_empty_public_key_fails_closed() {
    let config = config(PREVIEW, STABLE).replace(KEY, "");
    assert!(Trust::parse(&config).is_err());
}

#[test]
fn malformed_configuration_is_a_typed_error() {
    for json in ["", "{}", "{\"feeds\":[]}", "not json"] {
        assert!(Trust::parse(json).is_err(), "parsed {json:?}");
    }
}
