// ABOUTME: Renders the intentional-shutdown card in the isolated native fixture.
// ABOUTME: Captures narrow and wide windows without dialing a personal daemon.
use super::*;
use sheprd_client::{ClientEvent, ConnectTarget, protocol::ServerMessage};

pub(super) async fn verify(
    handle: WindowHandle<SheprdWindow>,
    light: bool,
    cx: &mut AsyncApp,
) -> Result<()> {
    handle.update(cx, |view, _, cx| -> Result<()> {
        let endpoint = &mut view.endpoints[view.selected_endpoint];
        // Apply shutdown before polling, so this Local label can never start Herdr.
        endpoint.connection.target = ConnectTarget::Local;
        {
            let mut inbox = endpoint
                .connection
                .inbox
                .lock()
                .map_err(|_| anyhow!("native shutdown inbox is poisoned"))?;
            inbox.apply(ClientEvent::Message(ServerMessage::ServerShutdown {
                reason: None,
            }));
            inbox.apply(ClientEvent::Disconnected {
                reason: "fixture shutdown".into(),
                ssh: None,
            });
        }
        view.poll_endpoints(cx);
        if !view.endpoints[view.selected_endpoint].herdr_stopped() || !view.reconnecting() {
            bail!("native shutdown fixture did not show restart advice");
        }
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
                bail!("shutdown capture root must be absolute");
            }
            let theme = if light { "light" } else { "dark" };
            let path = root.join(format!("shutdown-{theme}-{width:.0}.png"));
            cx.background_spawn(async move { pixels.save(path) })
                .await?;
        }
    }
    eprintln!("SHUTDOWN native PASS: restart advice at two sizes");
    Ok(())
}
