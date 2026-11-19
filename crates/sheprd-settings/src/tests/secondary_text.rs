// ABOUTME: Measures secondary text against the shipped light themes' chrome colors.
// ABOUTME: Keeps the text contrast floor independent of the optional high-contrast mode.
use super::*;
use crate::contrast::{Contrast, ratio};

#[test]
fn shipped_light_theme_secondary_text_meets_the_text_floor() {
    let mut failures = Vec::new();
    for name in [
        "GitHub Light Default",
        "Atom One Light",
        "Gruvbox Light",
        "One Half Light",
    ] {
        let config = Config {
            theme: name.into(),
            contrast: Contrast::Standard,
            ..Default::default()
        };
        let theme = config.theme(false).unwrap();
        for (label, color) in [("muted", theme.muted), ("subtext", theme.subtext())] {
            for (surface, background) in [
                ("background", theme.background),
                ("surface", theme.surface),
                ("hover", theme.active),
            ] {
                let measured = ratio(color, background);
                if measured < 4.5 {
                    failures.push(format!("{name} {label} on {surface}: {measured:.3}:1"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
