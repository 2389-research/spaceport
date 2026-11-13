// ABOUTME: Checks native local-control actions without a profile or personal daemon.
// ABOUTME: Captures fresh window frames and verifies popup input and focus isolation.
use super::*;
use crate::menu::Page;

pub(super) async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    let light = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_LIGHT").is_some();
    handle.update(cx, |view, window, cx| -> Result<()> {
        view.focus.focus(window, cx);
        view.theme = if light {
            view.config.theme_named("Catppuccin Latte", true)? // i18n: data
        } else {
            crate::config::Theme::default()
        };
        view.theme_light = light;
        cx.notify();
        Ok(())
    })??;
    for width in [800., 360.] {
        handle.update(cx, |_, window, _| window.resize(size(px(width), px(600.))))?;
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        let capture = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE");
        let pixels = AnyWindowHandle::from(handle).update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            capture
                .as_ref()
                .map(|_| window.render_to_image())
                .transpose()
        })??;
        if let (Some(root), Some(pixels)) = (capture, pixels) {
            let root = std::path::PathBuf::from(root);
            if !root.is_absolute() {
                bail!("local-controls capture root must be absolute");
            }
            let theme = if light { "light" } else { "dark" };
            let path = root.join(format!("{theme}-{width:.0}.png"));
            cx.background_spawn(async move { pixels.save(&path) })
                .await?;
        }
        for (command, expected) in [
            (Command::Keybinds, Page::Keybinds),
            (Command::Themes, Page::Themes),
            (Command::About, Page::About),
        ] {
            AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<()> {
                let view = root
                    .downcast::<SheprdWindow>()
                    .map_err(|_| anyhow!("unexpected root"))?;
                view.update(cx, |view, cx| view.command(command, window, cx));
                window.refresh();
                window.draw(cx).clear(cx);
                let before = view.read(cx).input_probe;
                if view.read(cx).menu.page != Some(expected) {
                    bail!("native local action {command:?} did not open {expected:?}");
                }
                view.update(cx, |view, cx| view.command(Command::Tab, window, cx));
                window.dispatch_keystroke(crate::keymap::parse_keystroke("escape")?, cx);
                let state = view.read(cx);
                if state.menu.page.is_some()
                    || state.input_probe.text != before.text
                    || state.input_probe.keys != before.keys
                    || state.input_probe.actions != before.actions
                    || !state.focus.is_focused(window)
                {
                    bail!("native local action {command:?} leaked input or lost focus");
                }
                Ok(())
            })??;
        }
    }
    about::verify(handle, cx).await?;
    #[cfg(target_os = "macos")]
    crate::settings_window::capture_branding(handle, cx).await?;
    if !cx
        .background_spawn(async { crate::diagnostics::flush(Duration::from_secs(2)) })
        .await
    {
        bail!("native local-controls diagnostics did not flush");
    }
    eprintln!("LOCAL CONTROLS native PASS: local actions, popup isolation and focus at two sizes");
    Ok(())
}
