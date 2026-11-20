// ABOUTME: Checks held Enter against actual native confirmation windows.
// ABOUTME: Uses inert fixtures with no process worker, external host, or Core connection.
use super::*;
use std::sync::Arc;

pub(super) async fn verify(
    handle: WindowHandle<SheprdWindow>,
    light: bool,
    cx: &mut AsyncApp,
) -> Result<()> {
    handle.update(cx, |view, _, _| -> Result<()> {
        let snapshot = serde_json::from_str(include_str!(
            "../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
        ))?;
        view.live.snapshot = Some(Arc::new(snapshot));
        Ok(())
    })??;
    for width in [800., 360.] {
        for name in ["processes", "device", "core-pane"] {
            let tab = handle.update(cx, |view, window, cx| -> Result<_> {
                view.config.confirm_close_pane = true;
                window.resize(size(px(width), px(600.)));
                let mut tab = None;
                let ready = match name {
                    "processes" => view.prepare_process_confirmation_check(window, cx),
                    "device" => view.prepare_host_confirmation_check(window, cx),
                    _ => {
                        tab = view.prepare_core_close_confirmation_check(window, cx);
                        tab.is_some()
                    }
                };
                if !ready {
                    bail!("could not prepare native {name} confirmation");
                }
                Ok(tab)
            })??;
            AnyWindowHandle::from(handle).update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
            })?;
            cx.background_executor()
                .timer(crate::motion::duration(crate::motion::ENTER) + Duration::from_millis(30))
                .await;
            let capture = std::env::var_os("HERDR_TEST_LOCAL_CONTROLS_CAPTURE");
            let pixels =
                AnyWindowHandle::from(handle).update(cx, |_, window, cx| -> Result<_> {
                    window.refresh();
                    window.draw(cx).clear(cx);
                    window.dispatch_event(
                        PlatformInput::KeyDown(KeyDownEvent {
                            keystroke: crate::keymap::parse_keystroke("enter")?,
                            is_held: true,
                            prefer_character_input: false,
                        }),
                        cx,
                    );
                    window.draw(cx).clear(cx);
                    capture
                        .as_ref()
                        .map(|_| window.render_to_image())
                        .transpose()
                })??;
            handle.update(cx, |view, _, _| -> Result<()> {
                let untouched = match name {
                    "processes" => view.process_confirmation_untouched(),
                    "device" => view.host_confirmation_untouched(),
                    _ => tab.is_some_and(|tab| view.core_close_confirmation_untouched(tab)),
                };
                if !untouched {
                    bail!("held Enter submitted native {name} confirmation");
                }
                Ok(())
            })??;
            if let (Some(root), Some(pixels)) = (capture, pixels) {
                let theme = if light { "light" } else { "dark" };
                let file = std::path::PathBuf::from(root)
                    .join(format!("confirm-{name}-{theme}-{width:.0}.png"));
                cx.background_spawn(async move { pixels.save(file) })
                    .await?;
            }
            AnyWindowHandle::from(handle).update(cx, |_, window, cx| -> Result<()> {
                window.dispatch_keystroke(crate::keymap::parse_keystroke("escape")?, cx);
                Ok(())
            })??;
            handle.update(cx, |view, window, cx| -> Result<()> {
                let expected = (name == "processes").then_some(Page::PaneProcesses);
                if view.menu.page != expected {
                    bail!("native {name} confirmation did not cancel");
                }
                view.dismiss_menu(window, cx);
                if let Some(tab) = tab
                    && !view.remove_core_close_confirmation_check(tab, cx)
                {
                    bail!("native Core confirmation fixture did not clean up");
                }
                Ok(())
            })??;
        }
    }
    eprintln!("CONFIRMATIONS native PASS: held Enter and cancellation at two sizes");
    Ok(())
}
