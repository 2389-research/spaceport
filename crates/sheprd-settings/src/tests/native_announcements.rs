// ABOUTME: Validates the real configurations consumed by the native announcement scenario.
// ABOUTME: Keeps stage names distinct in parser diagnostics so old events cannot qualify Zen.
use super::*;

#[test]
fn native_announcement_settings_and_failure_names_reach_their_consumers() {
    let config = Config::parse(include_str!(
        "../../../sheprd-gpui/tests/support/announcement-config.toml"
    ))
    .expect("native announcement settings");
    assert!(config.features.qa_menu);
    assert!(!config.status_bar.show);
    let failures = [
        (
            "native_hidden",
            include_str!("../../../sheprd-gpui/tests/support/announcement-hidden-invalid.toml"),
        ),
        (
            "native_zen",
            include_str!("../../../sheprd-gpui/tests/support/announcement-zen-invalid.toml"),
        ),
    ];
    for (index, (stage, input)) in failures.iter().enumerate() {
        let error = Config::parse(input)
            .expect_err("malformed announcement settings")
            .to_string();
        assert!(error.contains(stage), "{error}");
        assert!(!error.contains(failures[1 - index].0), "{error}");
    }
}
