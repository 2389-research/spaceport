// ABOUTME: Caches OS accessibility preferences for animation and theme consumers.
// ABOUTME: Applies one coherent snapshot without blocking the UI on portal calls.
use gpui::{App, Global, Task};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

#[cfg(any(target_os = "linux", all(test, unix)))]
mod portal;
#[cfg(test)]
pub(crate) mod tests;

const REDUCE_MOTION: u8 = 1;
const HIGH_CONTRAST: u8 = 2;
static POLICY: AtomicU8 = AtomicU8::new(0);

/// Missing preferences leave the last known policy unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Preferences {
    pub(crate) high_contrast: Option<bool>,
    pub(crate) reduce_motion: Option<bool>,
}

#[cfg(test)]
thread_local! {
    // Injected UI tests must not change another test's background palette reads.
    static OVERRIDE: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

fn snapshot() -> u8 {
    #[cfg(test)]
    if let Some(policy) = OVERRIDE.get() {
        return policy;
    }
    POLICY.load(Ordering::Relaxed)
}

fn store(preferences: Preferences) -> bool {
    let previous = snapshot();
    let mut policy = previous;
    for (flag, value) in [
        (REDUCE_MOTION, preferences.reduce_motion),
        (HIGH_CONTRAST, preferences.high_contrast),
    ] {
        if let Some(value) = value {
            policy = if value { policy | flag } else { policy & !flag };
        }
    }
    #[cfg(test)]
    if OVERRIDE.get().is_some() {
        OVERRIDE.set(Some(policy));
        return (previous ^ policy) & HIGH_CONTRAST != 0;
    }
    POLICY.store(policy, Ordering::Relaxed);
    (previous ^ policy) & HIGH_CONTRAST != 0
}

pub(crate) fn reduce_motion() -> bool {
    snapshot() & REDUCE_MOTION != 0
}

pub(crate) fn high_contrast() -> bool {
    snapshot() & HIGH_CONTRAST != 0
}

/// Native getters run before startup palettes load. Linux resolves its portal asynchronously.
pub(crate) fn prime() {
    store(native());
}

/// This UI-thread owner updates GPUI and the shared policy together.
fn apply(preferences: Preferences, cx: &mut App) {
    let contrast_changed = store(preferences);
    cx.set_reduce_motion(reduce_motion());
    if contrast_changed {
        let mut main_window = false;
        for handle in cx.windows() {
            if let Some(handle) = handle.downcast::<crate::SheprdWindow>() {
                main_window = true;
                let _ = handle.update(cx, |view, _, cx| {
                    if !view.refresh_picker_contrast(cx) {
                        if crate::settings_window::theme_appearance_pending(cx) {
                            crate::settings_window::apply_theme_draft(
                                &mut view.config,
                                &mut view.theme,
                                cx,
                            );
                            crate::log_window::set_appearance(&view.config, &view.theme, cx);
                        } else {
                            view.apply_shared_theme(cx);
                            view.reload_theme(cx);
                        }
                    }
                    cx.notify();
                });
            }
        }
        let settings_window = crate::settings_window::refresh_system_contrast(cx);
        if !main_window && !settings_window {
            crate::log_window::refresh_system_contrast(cx);
        }
        cx.refresh_windows();
    }
}

struct Watcher {
    _task: Task<()>,
}
impl Global for Watcher {}

pub(crate) fn install(cx: &mut App) {
    cx.set_reduce_motion(reduce_motion());
    let task = cx.spawn(async move |cx| {
        loop {
            #[cfg(target_os = "linux")]
            let preferences = {
                let executor = cx.background_executor().clone();
                cx.background_executor()
                    .spawn(async move {
                        portal::with_deadline(
                            portal::read(),
                            executor.timer(Duration::from_secs(2)),
                        )
                        .await
                    })
                    .await
            };
            #[cfg(not(target_os = "linux"))]
            let preferences = native();
            cx.update(|cx| apply(preferences, cx));
            cx.background_executor().timer(Duration::from_secs(2)).await;
        }
    });
    cx.set_global(Watcher { _task: task });
}

#[cfg(target_os = "macos")]
fn native() -> Preferences {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    Preferences {
        high_contrast: Some(workspace.accessibilityDisplayShouldIncreaseContrast()),
        reduce_motion: Some(workspace.accessibilityDisplayShouldReduceMotion()),
    }
}

#[cfg(windows)]
fn native() -> Preferences {
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};
    Preferences {
        high_contrast: AccessibilitySettings::new()
            .and_then(|settings| settings.HighContrast())
            .ok(),
        reduce_motion: UISettings::new()
            .and_then(|settings| settings.AnimationsEnabled())
            .ok()
            .map(|enabled| !enabled),
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
fn native() -> Preferences {
    Preferences::default()
}
