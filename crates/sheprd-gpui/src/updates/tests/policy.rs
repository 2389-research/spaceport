// ABOUTME: Exercises preview identity and trust configuration eligibility.
// ABOUTME: Rejects source builds and each mismatched bundle setting.
use super::*;

#[path = "feed_expiry.rs"]
mod feed_expiry;

const FEED: &str = "https://example.test/appcast.xml";
const KEY: &str = "public-key";

// These tests pin the identity strings on purpose: the policy itself holds none.
const PREVIEW_ID: &str = "ai.2389.spaceport";
const DEVELOPMENT_ID: &str = "spaceport.development";

fn preview() -> Bundle<'static> {
    Bundle {
        id: PREVIEW_ID,
        version: "0.1.1",
        path: "/Applications/Spaceport.app",
        feed: FEED,
        key: KEY,
        signed_feed: true,
        signed_feed_failure_expiration: Some(0.0),
        verify_before_extraction: true,
        automatic_install: false,
        automatic_checks_configured: false,
    }
}

fn preview_build() -> Build<'static> {
    Build {
        channel: Channel::MacosPreview,
        id: PREVIEW_ID,
        version: "0.1.1",
        worktree: "0",
    }
}

fn accepts(bundle: &Bundle<'_>) -> bool {
    eligible(bundle, &preview_build(), FEED, KEY)
}

#[test]
fn matching_preview_is_eligible() {
    assert!(accepts(&preview()));
}

/// The decisions the string-matching policy made, kept through the move to a
/// typed channel. `"dev"` was the development build's version, so those rows
/// are development-channel builds now.
#[test]
fn eligibility_decisions_are_unchanged() {
    let development_bundle = Bundle {
        id: DEVELOPMENT_ID,
        ..preview()
    };
    let dev_version_bundle = Bundle {
        version: "dev",
        ..preview()
    };
    let other_bundle = Bundle {
        id: "ai.2389.spaceport.other",
        ..preview()
    };
    let development = |id, version| Build {
        channel: Channel::Development,
        id,
        version,
        worktree: "0",
    };
    let rows = [
        (
            "development id",
            &development_bundle,
            development(DEVELOPMENT_ID, "0.1.1"),
            false,
        ),
        ("preview id", &preview(), preview_build(), true),
        (
            "mismatched bundle id",
            &other_bundle,
            preview_build(),
            false,
        ),
        (
            "dev version, bundle agrees",
            &dev_version_bundle,
            development(PREVIEW_ID, "dev"),
            false,
        ),
        (
            "dev version, bundle differs",
            &preview(),
            development(PREVIEW_ID, "dev"),
            false,
        ),
        (
            "worktree build",
            &preview(),
            Build {
                worktree: "1",
                ..preview_build()
            },
            false,
        ),
    ];
    for (name, bundle, build, expected) in rows {
        assert_eq!(eligible(bundle, &build, FEED, KEY), expected, "{name}");
    }
}

#[test]
fn development_channel_is_ineligible_whatever_its_identity_says() {
    // Even the preview's own ID and version, on a bundle that matches them.
    let build = Build {
        channel: Channel::Development,
        ..preview_build()
    };
    assert!(!build.can_update());
    assert!(!eligible(&preview(), &build, FEED, KEY));
    let build = Build {
        channel: Channel::Development,
        id: DEVELOPMENT_ID,
        version: "dev",
        worktree: "0",
    };
    let bundle = Bundle {
        id: DEVELOPMENT_ID,
        version: "dev",
        ..preview()
    };
    assert!(!eligible(&bundle, &build, FEED, KEY));
}

#[test]
fn preview_channel_follows_the_built_id_not_a_literal() {
    // The ID file may change: the bundle must match whatever was built.
    let build = Build {
        id: "ai.2389.renamed",
        ..preview_build()
    };
    assert!(!eligible(&preview(), &build, FEED, KEY));
    let bundle = Bundle {
        id: "ai.2389.renamed",
        ..preview()
    };
    assert!(eligible(&bundle, &build, FEED, KEY));
    assert!(!eligible(&bundle, &preview_build(), FEED, KEY));
}

#[test]
fn this_build_updates_only_as_a_preview_outside_a_worktree() {
    let build = Build::COMPILED;
    assert_eq!(build.id, crate::constants::APP_ID);
    assert_eq!(build.version, crate::constants::APP_VERSION);
    assert_eq!(
        build.can_update(),
        crate::constants::CHANNEL == Channel::MacosPreview && env!("HERDR_BUILD_WORKTREE") == "0"
    );
    let bundle = Bundle {
        id: build.id,
        version: build.version,
        ..preview()
    };
    assert_eq!(eligible(&bundle, &build, FEED, KEY), build.can_update());
}

#[test]
fn mismatched_bundle_or_trust_settings_are_ineligible() {
    for change in 0..10 {
        let mut bundle = preview();
        match change {
            0 => bundle.id = DEVELOPMENT_ID,
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
        let build = Build {
            worktree: marker,
            ..preview_build()
        };
        assert!(
            !eligible(&preview(), &build, FEED, KEY),
            "accepted build marker {marker:?}"
        );
    }
}
