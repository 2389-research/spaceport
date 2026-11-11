// ABOUTME: Exercises preview identity and trust configuration eligibility.
// ABOUTME: Rejects source builds and each mismatched bundle setting.
use super::*;

#[path = "feed_expiry.rs"]
mod feed_expiry;

fn preview() -> Bundle<'static> {
    Bundle {
        id: "ai.2389.spaceport",
        version: "0.1.1",
        path: "/Applications/Spaceport.app",
        feed: "https://example.test/appcast.xml",
        key: "public-key",
        signed_feed: true,
        signed_feed_failure_expiration: Some(0.0),
        verify_before_extraction: true,
        automatic_install: false,
        automatic_checks_configured: false,
    }
}

fn accepts(bundle: &Bundle<'_>) -> bool {
    eligible(
        bundle,
        "ai.2389.spaceport",
        "0.1.1",
        "https://example.test/appcast.xml",
        "public-key",
        "0",
    )
}

#[test]
fn matching_preview_is_eligible() {
    assert!(accepts(&preview()));
}

#[test]
fn development_build_is_ineligible() {
    assert!(!eligible(
        &preview(),
        "spaceport.development",
        "0.1.1",
        "https://example.test/appcast.xml",
        "public-key",
        "0",
    ));
    assert!(!eligible(
        &preview(),
        "ai.2389.spaceport",
        "dev",
        "https://example.test/appcast.xml",
        "public-key",
        "0",
    ));
}

#[test]
fn mismatched_bundle_or_trust_settings_are_ineligible() {
    for change in 0..10 {
        let mut bundle = preview();
        match change {
            0 => bundle.id = "spaceport.development",
            1 => bundle.version = "0.1.0",
            2 => bundle.path = "/tmp/spaceport",
            3 => bundle.feed = "https://attacker.test/feed",
            4 => bundle.key = "other-key",
            5 => bundle.signed_feed = false,
            6 => bundle.verify_before_extraction = false,
            7 => bundle.automatic_install = true,
            8 => bundle.automatic_checks_configured = true,
            _ => bundle.version = "dev",
        }
        assert!(!accepts(&bundle), "accepted mismatch {change}");
    }
}

#[test]
fn preview_identity_cannot_override_worktree_or_unknown_build_marker() {
    for marker in ["1", "", "false", "invalid"] {
        assert!(
            !eligible(
                &preview(),
                "ai.2389.spaceport",
                "0.1.1",
                "https://example.test/appcast.xml",
                "public-key",
                marker
            ),
            "accepted build marker {marker:?}"
        );
    }
}
