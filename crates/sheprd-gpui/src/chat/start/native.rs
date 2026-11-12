// ABOUTME: Paint chat-start failures through the real native flash renderer.
// ABOUTME: Check focus and capture private fixture windows across sizes and locales.
use super::Error;
use crate::{SheprdWindow, i18n::Localize};
use anyhow::{Context as _, Result, bail};
use gpui::{AnyWindowHandle, App, AppContext as _, AsyncApp, WindowHandle, px, size};
use sheprd_i18n::{Language, Locale, Pseudo};
use std::{path::PathBuf, time::Duration};

pub(crate) fn start(handle: WindowHandle<SheprdWindow>, cx: &mut App) {
    let capture = std::env::var_os("HERDR_TEST_CHAT_START_CAPTURE").map(PathBuf::from);
    cx.spawn(async move |cx| match verify(handle, capture, cx).await {
        Ok(count) => {
            eprintln!("CHAT START native PASS: {count} flashes; focus preserved"); // i18n: data
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("CHAT START native FAIL: {error:#}");
            std::process::exit(1);
        }
    })
    .detach();
}

async fn verify(
    handle: WindowHandle<SheprdWindow>,
    capture: Option<PathBuf>,
    cx: &mut AsyncApp,
) -> Result<usize> {
    if capture.as_ref().is_some_and(|path| !path.is_absolute()) {
        bail!("chat-start capture directory must be absolute");
    }
    let mut count = 0;
    for (pseudo, locale_name) in [
        (None, "en-US"),
        (Some(Pseudo::Accented), "en-XA"),
        (Some(Pseudo::Bidi), "en-XB"),
    ] {
        handle.update(cx, |_, _, cx| {
            crate::i18n::install(&Language::Locale(Locale::FALLBACK), cx);
            if let Some(pseudo) = pseudo {
                crate::i18n::toggle_pseudo(pseudo, cx);
            }
        })?;
        for light in [false, true] {
            let theme = if light { "light" } else { "dark" };
            for width in [480., 900.] {
                for (name, error) in [
                    ("disconnected", Error::NotConnected),
                    ("missing", Error::ProgramMissing),
                    ("pending", Error::ProgramUnknown),
                ] {
                    let message = error.localize();
                    handle.update(cx, |view, window, cx| -> Result<()> {
                        window.resize(size(px(width), px(460.)));
                        view.theme = if light {
                            view.config.theme_named("Catppuccin Latte", true)? // i18n: data
                        } else {
                            crate::config::Theme::default()
                        };
                        view.theme_light = light;
                        window.focus(&view.focus, cx);
                        view.flash_chat_error(&error, cx);
                        Ok(())
                    })??;
                    cx.background_executor()
                        .timer(Duration::from_millis(200))
                        .await;
                    handle.update(cx, |view, window, _| -> Result<()> {
                        let (flash, _) = view.flash_shown().context("chat flash did not paint")?;
                        if !flash.text.contains(message.as_str()) {
                            bail!("chat flash lost its localized diagnostic");
                        }
                        if !view.focus.is_focused(window) {
                            bail!("chat flash stole native input focus");
                        }
                        if (f32::from(window.viewport_size().width) - width).abs() > 1. {
                            bail!("native window did not reach requested width {width}");
                        }
                        Ok(())
                    })??;
                    // Capture reads the last painted scene, not the current model.
                    AnyWindowHandle::from(handle).update(cx, |_, window, cx| {
                        window.refresh();
                        window.draw(cx).clear(cx);
                    })?;
                    if let Some(directory) = &capture {
                        let path =
                            directory.join(format!("{locale_name}-{theme}-{width}-{name}.png"));
                        let pixels =
                            handle.update(cx, |_, window, _| window.render_to_image())??;
                        cx.background_spawn(async move { pixels.save(path) })
                            .await
                            .context("saving native chat-start screenshot")?;
                    }
                    eprintln!(
                        "CHAT START native case locale={locale_name} theme={theme} width={width} error={name} text={message}" // i18n: data
                    );
                    count += 1;
                }
            }
        }
    }
    handle.update(cx, |_, _, cx| {
        crate::i18n::install(&Language::Locale(Locale::FALLBACK), cx);
    })?;
    Ok(count)
}
