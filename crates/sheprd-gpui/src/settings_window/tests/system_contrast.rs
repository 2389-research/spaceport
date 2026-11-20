// ABOUTME: Checks OS contrast changes against raw Settings palettes and drafts.
// ABOUTME: Keeps the selected theme and restores its original colors on reset.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::accessibility::policy::{Preferences, tests as policy};
use crate::contrast::Contrast;

#[gpui::test]
fn system_contrast_restores_the_raw_selected_palette(cx: &mut TestAppContext) {
    check_contrast(cx, None);
}

#[cfg(unix)]
#[gpui::test]
fn system_contrast_preserves_an_unapplied_runtime_theme_draft(cx: &mut TestAppContext) {
    check_contrast(cx, Some(themes::Scope::Herdr));
}

#[gpui::test]
fn system_contrast_preserves_an_app_theme_draft(cx: &mut TestAppContext) {
    check_contrast(cx, Some(themes::Scope::App));
}

fn check_contrast(cx: &mut TestAppContext, draft: Option<themes::Scope>) {
    let _scope = policy::Scope::new();
    let source = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let weak = cx.update(|cx| {
        source
            .update(cx, |source, _, cx| {
                source.config.theme = "Nord".into();
                source.theme = Theme::builtin("Nord").unwrap();
                cx.weak_entity()
            })
            .unwrap()
    });
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = fixture_new(weak, cx);
        view.config.theme = "Nord".into();
        view.theme = Theme::builtin("Nord").unwrap();
        view
    });
    cx.update(|_, cx| {
        cx.default_global::<SettingsWindowHandle>().model = Some(view.downgrade());
    });
    if let Some(scope) = draft {
        view.update(cx, |view, cx| {
            view.shared =
                Some(herdr_settings::Settings::parse_text("[theme]\nname='nord'\n").unwrap());
            view.theme_io = Some(themes::ThemeIo {
                write: Arc::new(|_, _| panic!("OS contrast must not save a theme draft")),
                load: Arc::new(fixture_load),
                shared: None,
                resolve: None,
            });
            view.accept_theme_choice(
                themes::Choice {
                    scope,
                    name: if scope == themes::Scope::Herdr {
                        "dracula"
                    } else {
                        "Nord"
                    }
                    .into(),
                },
                cx,
            );
        });
    }
    let raw = Theme::builtin("Nord").unwrap();
    for high in [true, true, false] {
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
        cx.cx.update(|cx| {
            let contrast = if high {
                Contrast::High
            } else {
                Contrast::Standard
            };
            assert_eq!(
                source.read(cx).unwrap().theme,
                raw.clone().with_contrast(contrast)
            );
        });
        view.read_with(cx, |view, _| {
            assert_eq!(view.config.theme, "Nord");
            assert_eq!(view.theme_dirty(), draft.is_some());
            assert_eq!(
                view.theme,
                raw.clone().with_contrast(if high {
                    Contrast::High
                } else {
                    Contrast::Standard
                })
            );
        });
    }
}
