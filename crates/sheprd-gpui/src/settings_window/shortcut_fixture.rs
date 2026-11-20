// ABOUTME: Exposes the actual owned Settings window to Linux shortcut qualification.
// ABOUTME: Uses the normal close path without exposing the private Settings model.
use super::SettingsWindowHandle;
use anyhow::{Context as _, Result};
use gpui::{AnyWindowHandle, App};

pub(crate) fn window(cx: &App) -> Option<AnyWindowHandle> {
    let handle = cx.try_global::<SettingsWindowHandle>()?.window?;
    let handle = AnyWindowHandle::from(handle);
    cx.windows().contains(&handle).then_some(handle)
}

pub(crate) fn active(cx: &mut App) -> Result<bool> {
    let Some(handle) = cx
        .try_global::<SettingsWindowHandle>()
        .and_then(|state| state.window)
    else {
        return Ok(false);
    };
    handle
        .update(cx, |_, window, _| window.is_window_active())
        .context("observing the owned Settings window")
}

pub(crate) fn request_close(cx: &mut App) -> Result<()> {
    let handle = cx
        .try_global::<SettingsWindowHandle>()
        .and_then(|state| state.window)
        .context("owned Settings window is missing before close")?;
    // Normal closing may update source views; release the observer's root borrow first.
    cx.defer(move |cx| {
        if let Err(error) = handle.update(cx, |view, window, cx| view.close(window, cx)) {
            eprintln!("LINUX_SETTINGS_CLOSE_ERROR {error:#}");
        }
    });
    Ok(())
}
