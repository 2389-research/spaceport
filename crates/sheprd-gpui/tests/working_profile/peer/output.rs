// ABOUTME: Defines the private Windows TUI output contract used by the profile harness.
// ABOUTME: Keeps progress and completion observations free of terminal content.
use super::Targets;
use std::{path::Path, time::Instant};

#[derive(Clone, Copy, Debug)]
pub(in super::super) struct Progress {
    pub(in super::super) last_step: u8,
    pub(in super::super) complete: bool,
}

#[derive(Debug)]
pub(in super::super) struct Receipt {
    pub(in super::super) final_step: u8,
    pub(in super::super) complete: bool,
}

#[derive(Debug, thiserror::Error)]
pub(in super::super) enum Error {
    // keep-sorted start
    #[error("bounded TUI output producer is unavailable")]
    Unavailable,
    // keep-sorted end
}

pub(in super::super) struct Producer {}

impl Producer {
    /// Observe the command's first generated step by the absolute output deadline.
    pub(in super::super) fn start(
        _socket: &Path,
        _targets: &Targets,
        _deadline: Instant,
    ) -> Result<(Self, Progress), Error> {
        Err(Error::Unavailable)
    }

    /// Receive later observed progress. Fail at the earlier of the two deadlines.
    pub(in super::super) fn pump(&mut self, _until: Instant) -> Result<Progress, Error> {
        Err(Error::Unavailable)
    }

    /// Observe natural completion by the same absolute output deadline.
    pub(in super::super) fn finish(self) -> Result<Receipt, Error> {
        Err(Error::Unavailable)
    }
}
