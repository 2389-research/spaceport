// ABOUTME: Captures the real Fonts specimen in an isolated native fixture.
// ABOUTME: Keeps branding screenshots independent of desktop input and personal settings.
use super::*;

pub(crate) async fn capture(source: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    let Some(root) = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE") else {
        return Ok(());
    };
    let root = std::path::PathBuf::from(root);
    ensure!(root.is_absolute(), "Fonts capture root must be absolute");
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
        settings.update(cx, |_, window, _| {
            window.resize(size(px(width), px(780.)));
        })?;
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        let image = AnyWindowHandle::from(settings).update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            window.render_to_image()
        })??;
        let path = root.join(format!("fonts-{theme}-{width:.0}.png"));
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
    settings.update(cx, |_, window, _| window.remove_window())?;
    Ok(())
}
