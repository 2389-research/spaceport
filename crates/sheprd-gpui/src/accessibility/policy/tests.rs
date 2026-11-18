// ABOUTME: Verifies OS policy projections without changing machine preferences.
// ABOUTME: Isolates injected snapshots from parallel UI and background tests.
use super::*;

pub(crate) struct Scope(Option<u8>);
impl Scope {
    pub(crate) fn new() -> Self {
        Self(OVERRIDE.replace(Some(0)))
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        OVERRIDE.set(self.0);
    }
}

pub(crate) fn set(preferences: Preferences, cx: &mut App) {
    apply(preferences, cx);
}

pub(crate) fn reduce(cx: &mut App) {
    apply(
        Preferences {
            high_contrast: None,
            reduce_motion: Some(true),
        },
        cx,
    );
}

#[gpui::test]
fn one_snapshot_updates_gpui_manual_motion_and_theme_contrast(cx: &mut gpui::TestAppContext) {
    let _scope = Scope::new();
    cx.update(|cx| {
        apply(
            Preferences {
                high_contrast: Some(true),
                reduce_motion: Some(true),
            },
            cx,
        );
        assert!(cx.reduce_motion());
        assert!(reduce_motion());
        assert_eq!(
            crate::contrast::Contrast::Standard.or_system(),
            crate::contrast::Contrast::High
        );
        assert_eq!(
            crate::motion::progress(
                std::time::Instant::now(),
                std::time::Instant::now(),
                crate::motion::ENTER
            ),
            None
        );
        apply(
            Preferences {
                high_contrast: Some(false),
                reduce_motion: Some(false),
            },
            cx,
        );
        assert!(!cx.reduce_motion());
        assert_eq!(
            crate::contrast::Contrast::Standard.or_system(),
            crate::contrast::Contrast::Standard
        );
        assert_eq!(
            crate::contrast::Contrast::High.or_system(),
            crate::contrast::Contrast::High
        );
        let now = std::time::Instant::now();
        assert_eq!(
            crate::motion::progress(now, now, crate::motion::ENTER),
            Some(0.)
        );
    });
}

#[test]
fn missing_preferences_keep_the_last_snapshot_and_do_not_leak_to_other_threads() {
    let _scope = Scope::new();
    store(Preferences {
        high_contrast: Some(true),
        reduce_motion: Some(true),
    });
    store(Preferences::default());
    assert!(reduce_motion() && high_contrast());
    assert_eq!(
        std::thread::spawn(snapshot).join().ok(),
        Some(POLICY.load(Ordering::Relaxed))
    );
    store(Preferences {
        high_contrast: None,
        reduce_motion: Some(false),
    });
    assert!(!reduce_motion());
    assert!(high_contrast());
}

#[cfg(target_os = "macos")]
#[test]
fn native_probe_reads_the_safe_appkit_getters() {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    let preferences = native();
    assert_eq!(
        preferences.reduce_motion,
        Some(workspace.accessibilityDisplayShouldReduceMotion())
    );
    assert_eq!(
        preferences.high_contrast,
        Some(workspace.accessibilityDisplayShouldIncreaseContrast())
    );
}

#[cfg(windows)]
#[test]
fn native_probe_reads_the_safe_windows_getters() -> windows::core::Result<()> {
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};
    let preferences = native();
    assert_eq!(
        preferences.reduce_motion,
        Some(!UISettings::new()?.AnimationsEnabled()?)
    );
    assert_eq!(
        preferences.high_contrast,
        Some(AccessibilitySettings::new()?.HighContrast()?)
    );
    Ok(())
}
