// ABOUTME: Captures Fonts and settings descriptions in isolated native windows.
// ABOUTME: Keeps screenshots independent of desktop input and personal settings.
use super::*;

pub(crate) async fn capture(source: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    let Some(root) = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE") else {
        return Ok(());
    };
    let root = std::path::PathBuf::from(root);
    ensure!(root.is_absolute(), "Fonts capture root must be absolute");
    source.update(cx, |view, _, cx| view.load_shared_settings(cx))?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if source.update(cx, |view, _, _| -> Result<bool> {
            if let Some(error) = &view.settings.error {
                bail!("settings capture could not load view settings: {error}");
            }
            Ok(view.settings.loaded
                && view.settings.task.is_none()
                && view.settings.shared.is_some())
        })?? {
            break;
        }
        ensure!(Instant::now() < deadline, "settings capture load timed out");
        cx.background_executor()
            .timer(Duration::from_millis(10))
            .await;
    }
    let weak = source.update(cx, |_, _, cx| cx.entity().downgrade())?;
    cx.update(|cx| {
        open_with(weak, cx, |view, _, cx| {
            view.section = Section::Fonts;
            view.initialize_controls(cx);
            view.controls
                .set_active_face(crate::config::FontFace::Sidebar);
        });
    });
    let settings = cx
        .update(|cx| cx.default_global::<SettingsWindowHandle>().window)
        .context("Fonts capture window did not open")?;
    let light = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_LIGHT").is_some();
    let theme = if light { "light" } else { "dark" };
    for width in [960., 680.] {
        for (name, section, row) in [
            ("fonts", Section::Fonts, None),
            (
                "copy-on-select",
                Section::General,
                Some("general-copy-on-select"),
            ),
            ("notifications", Section::Notifications, None),
            (
                "notification-volume",
                Section::Notifications,
                Some("notif-volume"),
            ),
        ] {
            settings.update(cx, |_, window, _| {
                window.resize(size(px(width), px(780.)));
            })?;
            settings.update(cx, |view, _, cx| {
                cx.default_global::<search::RowBounds>().0.clear();
                view.section = section;
                view.body_scroll.set_offset(Point::default());
                cx.notify();
            })?;
            AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
            })?;
            if let Some(row) = row {
                settings.update(cx, |view, _, cx| view.scroll_to_row(row, cx))?;
            }
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            let image = AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                window.render_to_image()
            })??;
            let path = root.join(format!("{name}-{theme}-{width:.0}.png"));
            cx.background_spawn(async move {
                use image::codecs::png::{CompressionType, FilterType, PngEncoder};
                let mut png = Vec::new();
                image.write_with_encoder(PngEncoder::new_with_quality(
                    &mut png,
                    CompressionType::Fast,
                    FilterType::Sub,
                ))?;
                std::fs::write(path, png)?;
                anyhow::Ok(())
            })
            .await?;
        }
    }
    settings.update(cx, |_, window, _| window.remove_window())?;
    Ok(())
}
