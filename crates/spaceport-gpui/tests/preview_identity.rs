// ABOUTME: Verify the bounded signed macOS channels and unchanged development identity.
// ABOUTME: Run pure build selection without GPUI or signing credentials.
#[path = "../preview_identity.rs"]
mod preview_identity;
use preview_identity::{Identity, Requested};

const PREVIEW_VERSION: &str = "20261109.1";

#[test]
fn development_and_the_signed_channels_are_distinct() {
    assert_eq!(
        preview_identity::select(Requested::Development, "linux"),
        Some(Identity {
            channel: "development",
            app_id: "spaceport.development",
            version: "dev",
        })
    );
    for (requested, channel) in [
        (Requested::MacosPreview(PREVIEW_VERSION), "macos-preview"),
        (Requested::Production(PREVIEW_VERSION), "production"),
    ] {
        assert_eq!(
            preview_identity::select(requested, "macos"),
            Some(Identity {
                channel,
                app_id: "ai.2389.spaceport",
                version: PREVIEW_VERSION,
            })
        );
    }
}

#[test]
fn every_target_without_a_version_is_a_development_build() {
    for target in ["macos", "linux", "windows", ""] {
        let channel = preview_identity::select(Requested::Development, target)
            .map(|identity| identity.channel);
        assert_eq!(channel, Some(preview_identity::DEVELOPMENT));
        assert_ne!(channel, Some(preview_identity::MACOS_PREVIEW));
        assert_ne!(channel, Some(preview_identity::PRODUCTION));
    }
}

#[test]
fn signed_channels_reject_other_targets_and_malformed_versions() {
    for target in ["linux", "windows", ""] {
        assert!(
            preview_identity::select(Requested::MacosPreview(PREVIEW_VERSION), target).is_none()
        );
        assert!(preview_identity::select(Requested::Production(PREVIEW_VERSION), target).is_none());
    }
    for version in [
        "",
        "dev",
        "v20261109.1",
        "20261109.01",
        "20261109.0",
        "20261109.1\n",
        "20261109.1-rc1",
        "1.0.0",
    ] {
        assert!(
            preview_identity::select(Requested::MacosPreview(version), "macos").is_none(),
            "{version:?}"
        );
        assert!(
            preview_identity::select(Requested::Production(version), "macos").is_none(),
            "{version:?}"
        );
    }
}

#[test]
fn app_id_data_accepts_a_final_newline_and_rejects_invalid_identifiers() {
    assert_eq!(
        preview_identity::app_id("ai.2389.spaceport\n"),
        Some("ai.2389.spaceport")
    );
    for id in [
        "",
        "Spaceport",
        "ai..spaceport",
        "ai.space port",
        "ai.spaceport\ninjected",
        "ai.spaceport;",
    ] {
        assert_eq!(preview_identity::app_id(id), None);
    }
}
