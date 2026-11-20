// ABOUTME: Verifies shared tab presentation behavior through the layout helpers.
// ABOUTME: Keeps focus raster cases separate from tooltip text contracts.
use super::*;
use core::prelude::v1::test;

#[cfg(all(target_os = "macos", feature = "integration-test"))]
mod focus_raster;

#[test]
fn a_tooltip_standing_in_for_a_title_names_a_mute() {
    assert_eq!(hint("logs".into(), false), "logs");
    assert_eq!(
        hint("logs".into(), true),
        "\u{2068}logs\u{2069} (notifications muted)"
    );
}
