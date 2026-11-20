// ABOUTME: Observes bounded app-frame text only for an explicitly selected private Core fixture.
// ABOUTME: Uses the existing nonblocking log writer without changing input, layout, or rendering.
use super::CoreTerminalView;
use sheprd_backend::FrameData;
use std::sync::OnceLock;

#[path = "../../../../test-support/core_frame_scope.rs"]
mod scope;

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let nonce = std::env::var_os("SPACEPORT_TEST_CORE_FRAME_OBSERVER");
        let session = std::env::var_os("HC_SESSION");
        let socket = std::env::var_os("HC_SOCKET");
        scope::matches_target(nonce.as_deref(), session.as_deref(), socket.as_deref())
    })
}

fn sampled_rows(frame: &FrameData) -> Vec<(String, usize)> {
    frame
        .cells
        .chunks(usize::from(frame.width.max(1)))
        .take(3)
        .map(|row| {
            let text = row
                .iter()
                .take(96)
                .flat_map(|cell| cell.symbol.chars())
                .take(96)
                .collect();
            let hidden = row
                .iter()
                .take(96)
                .filter(|cell| cell.modifier & crate::terminal::HIDDEN != 0)
                .count();
            (text, hidden)
        })
        .collect()
}

pub(super) fn record(view: &CoreTerminalView) {
    if !enabled() {
        return;
    }
    let Some(frame) = &view.screen.frame else {
        return;
    };
    tracing::info!(
        terminal = %view.id,
        observe = view.observe,
        frame_width = frame.width,
        frame_height = frame.height,
        queued_size = ?view.last_queued_size,
        bounds = ?view.bounds,
        cell_width = view.look.cell_width,
        cell_height = view.look.cell_height,
        rows_with_hidden_counts = ?sampled_rows(frame),
        "CORE_FRAME_PROBE"
    );
}

#[cfg(test)]
#[path = "frame_probe/tests.rs"]
mod tests;
