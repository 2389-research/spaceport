// ABOUTME: Checks Ask's secondary text and source links against their actual page background.
// ABOUTME: Keeps normal text readable across the retained light and dark themes.
use super::*;

#[gpui::test]
fn disclosure_and_source_ink_meet_text_contrast(cx: &mut TestAppContext) {
    cx.update(|cx| {
        for name in ["Catppuccin Latte", "Catppuccin Mocha", "Nord", "Dracula"] {
            let Some(theme) = crate::config::Theme::builtin(name) else {
                panic!("retained theme {name}");
            };
            let view = cx.new(|cx| View::new(Default::default(), theme, cx));
            let view = view.read(cx);
            for color in [view.theme.muted, view.theme.primary()] {
                let ink = view.text_ink(color);
                assert!(
                    crate::contrast::ratio(ink, view.theme.background) >= 4.5,
                    "{name}: {ink:06x} text must contrast with its page"
                );
            }
        }
    });
}
