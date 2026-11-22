use super::*;
use core::prelude::v1::test;

#[test]
fn a_tooltip_standing_in_for_a_title_names_a_mute() {
    assert_eq!(hint("logs".into(), false), "logs");
    assert_eq!(
        hint("logs".into(), true),
        "\u{2068}logs\u{2069} (notifications muted)"
    );
}

#[test]
fn every_tab_layout_draws_split_panes() {
    for &mode in TabLayoutMode::ALL {
        let chrome = layout_for(mode).panes();
        assert!(chrome.gutter() >= 1., "{mode:?} needs a gutter to drag");
        assert!(chrome.veil() > 0, "{mode:?} must mark the panes not in use");
        let (top, bottom) = chrome.corners();
        assert!(top >= 0. && bottom >= 0.);
    }
}
