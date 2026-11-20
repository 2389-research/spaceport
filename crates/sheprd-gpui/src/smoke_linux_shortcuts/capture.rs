// ABOUTME: Starts the owned raw-byte producer before a native key case is ready.
// ABOUTME: Reads bounded correlated hex receipts from the real projected terminal frame.
use super::{SheprdWindow, cases, smoke};
use anyhow::{Context as _, Result};
use sheprd_client::protocol::{ClientKeyCode, ClientKeyKind, ClientPaneInputEvent};

pub(super) use cases::capture::{Capture, marker, read};

pub(super) fn screen(view: &SheprdWindow) -> Result<String> {
    Ok(smoke::screen_text(
        &view
            .live
            .surface
            .as_ref()
            .context("capture surface is missing")?
            .frame,
    ))
}

pub(super) fn start(view: &SheprdWindow, nonce: &str, id: &str, bytes: usize) -> Result<()> {
    let path =
        std::path::PathBuf::from(std::env::var_os("HOME").context("capture HOME is missing")?)
            .join(cases::CAPTURE_FILE);
    let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
    let command = format!(
        "/bin/sh {} {} {} {}",
        quote(path.to_str().context("capture path is not UTF-8")?),
        quote(nonce),
        quote(id),
        bytes
    );
    let snapshot = view
        .live
        .snapshot
        .as_ref()
        .context("capture snapshot is missing")?;
    let pane = snapshot
        .focused_pane_id
        .as_deref()
        .context("capture pane is missing")?;
    let handle = view
        .endpoints
        .get(view.selected_endpoint)
        .and_then(|endpoint| endpoint.connection.handle.as_ref())
        .context("capture connection is missing")?;
    // This authored setup finishes before READY; measured input comes only from Mutter.
    handle.send_input(
        &snapshot.boot_id,
        pane,
        [
            ClientPaneInputEvent::TextCommit(command),
            ClientPaneInputEvent::Key {
                code: ClientKeyCode::Enter,
                modifiers: 0,
                kind: ClientKeyKind::Press,
                repeat_count: 1,
                shifted_codepoint: None,
                generated_text: None,
                tracks_release: false,
                physical_key_id: None,
                windows_record: None,
            },
        ],
    )?;
    Ok(())
}
