//! Native Settings window checks run by the sidebar smoke driver.
use super::*;

pub(super) async fn sidebar_preferences(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
) -> Result<()> {
    handle.update(cx, |view, _, cx| view.load_shared_settings(cx))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let loaded = handle.update(cx, |view, _, _| -> Result<bool> {
            if let Some(error) = &view.settings.error {
                bail!("shared settings load: {error}");
            }
            Ok(view.settings.loaded
                && view.settings.task.is_none()
                && view.settings.shared.is_some())
        })??;
        if loaded {
            break;
        }
        if Instant::now() >= deadline {
            bail!("shared settings load timed out");
        }
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
    crate::settings_window::verify_native(handle, cx).await?;
    Ok(())
}
