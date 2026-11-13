// ABOUTME: Verify Spaceport and upstream attribution in the real native About box.
// ABOUTME: Check painted glyphs, narrow layouts and focus in the isolated GUI fixture.
use super::*;
use crate::about::{AUTHOR, COPYRIGHT_YEAR, PUBLISHER};
use crate::i18n::fl;
use crate::menu::Page;

pub(super) async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    use crate::sidebar::layout_tests::PaintedProbes;
    let capture = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE");
    for width in [800., 360.] {
        handle.update(cx, |_, window, _| window.resize(size(px(width), px(600.))))?;
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<()> {
            let view = root
                .downcast::<SheprdWindow>()
                .map_err(|_| anyhow!("unexpected root"))?;
            view.update(cx, |view, cx| view.open_about(window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
            Ok(())
        })??;
        // Capture the settled popup, after its actual enter animation starts.
        cx.background_executor()
            .timer(crate::motion::duration(crate::motion::ENTER))
            .await;
        let image =
            AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<_> {
                let view = root
                    .downcast::<SheprdWindow>()
                    .map_err(|_| anyhow!("unexpected root"))?;
                cx.default_global::<PaintedProbes>().0.clear();
                window.draw(cx).clear(cx);
                cx.default_global::<PaintedProbes>().check()?;
                for label in [
                    fl!(
                        "about-publisher",
                        name = crate::constants::WINDOW_TITLE,
                        publisher = PUBLISHER
                    ),
                    fl!(
                        "about-modifications",
                        name = crate::constants::WINDOW_TITLE,
                        copyright = COPYRIGHT_YEAR,
                        publisher = PUBLISHER
                    ),
                    fl!(
                        "about-original-copyright",
                        name = "Sheprd",
                        copyright = COPYRIGHT_YEAR,
                        author = AUTHOR
                    ),
                    fl!("about-notices"),
                ] {
                    let probe = cx
                        .global::<PaintedProbes>()
                        .0
                        .get(&label)
                        .with_context(|| format!("missing native About text {label}"))?;
                    // Wrapped text is valid; all source glyphs and the full layout must remain visible.
                    if probe.cached != label
                        || probe.glyph_text != label.replace(['\u{2068}', '\u{2069}'], "")
                        || probe.bounds.left() < probe.mask.left()
                        || probe.bounds.right() > probe.mask.right()
                        || probe.bounds.top() < probe.mask.top()
                        || probe.bounds.bottom() > probe.mask.bottom()
                        || probe.bounds.bottom() > window.viewport_size().height
                    {
                        bail!("native About text clipped or changed: {label}");
                    }
                }
                let image = capture
                    .as_ref()
                    .map(|_| window.render_to_image())
                    .transpose()?;
                let before = view.read(cx).input_probe;
                window.dispatch_keystroke(crate::keymap::parse_keystroke("escape")?, cx);
                let state = view.read(cx);
                if state.menu.page == Some(Page::About)
                    || state.input_probe.text != before.text
                    || state.input_probe.keys != before.keys
                    || !state.focus.is_focused(window)
                {
                    bail!("native About lost close/focus isolation");
                }
                Ok(image)
            })??;
        if let (Some(root), Some(image)) = (&capture, image) {
            let theme = if std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_LIGHT").is_some() {
                "light"
            } else {
                "dark"
            };
            let path = std::path::PathBuf::from(root).join(format!("about-{theme}-{width:.0}.png"));
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
    eprintln!(
        "ABOUT native PASS: Spaceport and upstream attribution glyphs, narrow layouts and focus"
    );
    Ok(())
}
