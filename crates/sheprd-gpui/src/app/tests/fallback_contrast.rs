// ABOUTME: Verifies readable startup colors when configuration loading fails.
// ABOUTME: Keeps fallback palettes subject to the same OS and text contrast rules.
use super::*;
use crate::accessibility::policy::{Preferences, tests as policy};
use crate::contrast::{Contrast, ratio};

#[gpui::test]
fn fallback_contrast_respects_os_preferences_and_readable_text(cx: &mut gpui::TestAppContext) {
    let _scope = policy::Scope::new();
    for high in [true, false] {
        cx.update(|cx| {
            policy::set(
                Preferences {
                    high_contrast: Some(high),
                    reduce_motion: None,
                },
                cx,
            )
        });
        let appearance = InitialAppearance::load(|| Err(crate::Error::MissingHome), false);
        assert!(appearance.error.is_some());
        assert_eq!(
            appearance.theme.contrast,
            if high {
                Contrast::High
            } else {
                Contrast::Standard
            }
        );
        for color in [appearance.theme.muted, appearance.theme.subtext()] {
            for background in [
                appearance.theme.background,
                appearance.theme.surface,
                appearance.theme.active,
            ] {
                assert!(ratio(color, background) >= 4.5);
            }
        }
    }
}
