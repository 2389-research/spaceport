// ABOUTME: Verify the retained native profile popup and absence of account sign-in.
// ABOUTME: Exercise painted menu rows, input isolation and focus at two window sizes.
use super::*;
use crate::i18n::fl;
use crate::menu::Page;

pub(super) async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    use crate::sidebar::layout_tests::PaintedProbes;
    for (width, height) in [(640., 400.), (1200., 780.)] {
        handle.update(cx, |_, window, _| {
            window.resize(size(px(width), px(height)));
        })?;
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<()> {
            let view = root
                .downcast::<SheprdWindow>()
                .map_err(|_| anyhow!("unexpected root"))?;
            view.update(cx, |view, cx| view.open_profile(window, cx));
            if view.read(cx).menu.page != Some(Page::Profile) {
                bail!("profile popup did not open");
            }
            cx.default_global::<PaintedProbes>().0.clear();
            window.draw(cx).clear(cx);
            cx.default_global::<PaintedProbes>().check()?;
            let probes = &cx.global::<PaintedProbes>().0;
            // Include the heading as text while keeping it out of keyboard actions.
            for label in [
                fl!("menu-page-profile"),
                fl!("menu-chrome-settings"),
                fl!("menu-chrome-shortcuts"),
                fl!("menu-chrome-themes"),
                fl!("menu-chrome-about"),
            ] {
                let probe = probes
                    .get(&label)
                    .with_context(|| format!("missing profile text {label}"))?;
                if probe.clipped || probe.bounds.bottom() > window.viewport_size().height {
                    bail!("profile text {label} clipped");
                }
            }
            if probes.keys().any(|label| {
                let lower = label.to_lowercase();
                lower.contains("github") || lower.contains("sign in")
            }) {
                bail!("profile popup exposed a removed account action");
            }
            let before = view.read(cx).input_probe;
            window.dispatch_action(
                Box::new(RunCommand {
                    command: Command::Tab,
                }),
                cx,
            );
            window.dispatch_keystroke(crate::keymap::parse_keystroke("down")?, cx);
            window.dispatch_keystroke(crate::keymap::parse_keystroke("escape")?, cx);
            let state = view.read(cx);
            if state.menu.page.is_some()
                || state.input_probe.text != before.text
                || state.input_probe.keys != before.keys
                || state.input_probe.actions != before.actions
                || !state.focus.is_focused(window)
            {
                bail!("profile popup leaked input or lost focus");
            }
            Ok(())
        })??;
    }
    about::verify(handle, cx).await?;
    eprintln!(
        "PROFILE native PASS: retained actions painted without clipping; no account sign-in; menu input isolated and focus restored at two sizes"
    );
    Ok(())
}
