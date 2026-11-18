// ABOUTME: Checks OS contrast when the paused log console is the only window.
// ABOUTME: Uses a built-in palette and prevents reads of private diagnostic files.
use super::*;
use crate::accessibility::policy::{Preferences, tests as policy};

#[gpui::test]
fn a_log_only_window_follows_system_contrast(cx: &mut TestAppContext) {
    let _scope = policy::Scope::new();
    let config = Config {
        theme: "Nord".into(),
        ..Default::default()
    };
    let raw = Theme::builtin("Nord").unwrap();
    cx.update(|cx| set_appearance(&config, &raw, cx));
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = LogWindow::new(window, cx);
        view.following = false;
        view.generation = Some(diagnostics::generation());
        view
    });
    for high in [true, false] {
        cx.cx.update(|cx| {
            policy::set(
                Preferences {
                    high_contrast: Some(high),
                    reduce_motion: None,
                },
                cx,
            )
        });
        cx.run_until_parked();
        view.read_with(cx, |view, _| {
            assert_eq!(
                view.appearance.theme,
                raw.clone().with_contrast(if high {
                    crate::contrast::Contrast::High
                } else {
                    crate::contrast::Contrast::Standard
                })
            );
            assert!(view.rows.is_empty());
        });
    }
}
