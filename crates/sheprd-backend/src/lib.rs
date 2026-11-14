#![doc = include_str!("../README.md")]

mod error;
mod terminal;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;

use std::time::Duration;

pub use error::{Error, Result};
pub use sheprd_protocol::{
    AgentStatus, ClientKeyCode, ClientKeyKind, ClientMouseButton, ClientMouseKind,
    ClientMousePosition, ClientPaneInputEvent, CursorState, FrameData, PaneSurfacePatchRow,
};
pub use terminal::{
    Change, Exit, KeyReporting, MAX_CELLS, MAX_SIDE, MAX_TERMINAL_ID, Role, RunState, Size, Spawn,
    SurfaceUpdate, TerminalId, TerminalInfo,
};

/// What attaches one host's terminals.
///
/// Moved to the worker thread that attaches, so implementations are
/// `Send + Sync`. `attach` may block on the host: never call it on the UI
/// thread.
pub trait Backend: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Terminal: Terminal<Error = Self::Error>;

    /// Shows `id` at `size`. An observer never types into or resizes it.
    fn attach(
        &self,
        id: &TerminalId,
        size: Size,
        role: Role,
    ) -> Result<Self::Terminal, Self::Error>;
}

/// One attached terminal, owned by the worker that drains it.
pub trait Terminal: Send {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Keys, text, pastes, and mouse reports, encoded for the program's
    /// current modes by whichever side emulates the terminal.
    fn input(&mut self, input: &ClientPaneInputEvent) -> Result<(), Self::Error>;

    fn resize(&mut self, size: Size) -> Result<(), Self::Error>;

    fn focus(&mut self, focused: bool) -> Result<(), Self::Error>;

    /// What changed, waiting at most `timeout`; empty when nothing did.
    fn next(&mut self, timeout: Duration) -> Result<Vec<SurfaceUpdate>, Self::Error>;
}
