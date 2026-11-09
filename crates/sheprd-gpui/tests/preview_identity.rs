// ABOUTME: Verify the bounded macOS preview mode and unchanged development identity.
// ABOUTME: Run pure build selection without GPUI or signing credentials.
#[path = "../preview_identity.rs"]
mod preview_identity;

#[test]
fn development_and_macos_preview_are_distinct() {
    assert_eq!(
        preview_identity::select(None, "linux"),
        Some(("spaceport.development", "dev"))
    );
    assert_eq!(
        preview_identity::select(Some("20261109.1"), "macos"),
        Some(("ai.2389.spaceport", "20261109.1"))
    );
}

#[test]
fn preview_rejects_other_targets_and_malformed_versions() {
    for target in ["linux", "windows", ""] {
        assert!(preview_identity::select(Some("20261109.1"), target).is_none());
    }
    for version in [
        "",
        "dev",
        "v20261109.1",
        "20261109.01",
        "20261109.0",
        "20261109.1\n",
        "20261109.1-rc1",
    ] {
        assert!(preview_identity::select(Some(version), "macos").is_none());
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
