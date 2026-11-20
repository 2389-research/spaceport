// ABOUTME: Observes bounded app-frame text and logical focus for a selected private Core fixture.
// ABOUTME: Uses the existing nonblocking log writer without changing input, layout, or rendering.
use super::CoreTerminalView;
use gpui::{App, Window};
use sheprd_backend::{FrameData, TerminalId};
use std::{cell::RefCell, sync::OnceLock};

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

pub(super) fn record_focus(view: &CoreTerminalView, window: &Window, cx: &App) {
    if !enabled() {
        return;
    }
    thread_local! {
        static PREVIOUS: RefCell<Option<(TerminalId, bool, bool, String)>> = const { RefCell::new(None) };
    }
    let state = (
        view.id.clone(),
        window.is_window_active(),
        view.focus.is_focused(window),
        format!("{:?}", window.focused(cx)),
    );
    let changed = PREVIOUS.with_borrow_mut(|previous| {
        if previous.as_ref() == Some(&state) {
            return false;
        }
        *previous = Some(state.clone());
        true
    });
    if changed {
        tracing::info!(
            terminal = %state.0,
            window_active = state.1,
            core_focused = state.2,
            current_focus = %state.3,
            core_focus = ?view.focus,
            "CORE_FOCUS_PROBE"
        );
    }
}

#[cfg(test)]
#[path = "frame_probe/tests.rs"]
mod tests;
