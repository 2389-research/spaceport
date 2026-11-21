// ABOUTME: Produces bounded output in one owned Windows TUI workspace.
// ABOUTME: Reports scalar progress from the exact pane's rendered markers.
use super::{Peer, Targets};
use crate::working_profile::report::MARKER;
use sheprd_client::{
    ClientEvent,
    protocol::{ClientShellSnapshot, FrameData, PaneSurfaceFrame, SurfaceRect},
};
use std::{io, path::Path, time::Instant};

const FINAL_STEP: u8 = 64;
const STEP_MS: u16 = 200;
const OUTPUT_TAG: &str = "SP_OUTPUT";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    #[error("the private TUI output boot changed")]
    BootChanged,
    #[error("the private TUI rejected the output command ({kind:?})")]
    CommandRejected { kind: io::ErrorKind },
    #[error("the private TUI output completion marker is out of order")]
    Completion,
    #[error("could not open the private TUI output connection ({kind:?})")]
    Connect { kind: io::ErrorKind },
    #[error("the private TUI output deadline expired")]
    Deadline,
    #[error("the private TUI output connection ended")]
    Disconnected,
    #[error("could not queue the private TUI output command ({kind:?})")]
    Input { kind: io::ErrorKind },
    #[error("the private TUI emitted a malformed output marker")]
    MalformedMarker,
    #[error("the private TUI output pane is missing, ambiguous, or no longer focused")]
    PaneChanged,
    #[error("the private TUI output pane rectangle is outside its surface")]
    PaneGeometry,
    #[error("the private TUI output regressed from step {previous} to {observed}")]
    Regression { previous: u8, observed: u8 },
    #[error("the private TUI output expected step {expected}, observed {observed}")]
    Sequence { expected: u8, observed: u8 },
    // keep-sorted end
}

pub(in super::super) struct Producer {
    peer: Peer,
    targets: Targets,
    deadline: Instant,
    progress: Progress,
}

impl Producer {
    /// Observe the command's first generated step by the absolute output deadline.
    pub(in super::super) fn start(
        socket: &Path,
        targets: &Targets,
        deadline: Instant,
    ) -> Result<(Self, Progress), Error> {
        let peer = Peer::begin(socket).map_err(|error| Error::Connect { kind: error.kind() })?;
        let mut producer = Self {
            peer,
            targets: targets.clone(),
            deadline,
            progress: Progress::default(),
        };
        loop {
            if let ClientEvent::Snapshot(snapshot) = producer.receive(deadline)? {
                producer.check_snapshot(&snapshot)?;
                break;
            }
        }
        // Queue one semantic input batch. Only a generated marker proves it arrived.
        producer
            .peer
            .send_line(targets, command())
            .map_err(|error| Error::Input { kind: error.kind() })?;
        let first = producer.pump(deadline)?;
        Ok((producer, first))
    }

    /// Receive later observed progress or return established completion.
    /// Fail at the earlier of the caller's and producer's deadlines.
    pub(in super::super) fn pump(&mut self, until: Instant) -> Result<Progress, Error> {
        let deadline = until.min(self.deadline);
        let previous = self.progress;
        while self.progress == previous && !self.progress.complete {
            match self.receive(deadline)? {
                ClientEvent::Snapshot(snapshot) => self.check_snapshot(&snapshot)?,
                ClientEvent::Surface(surface) => {
                    self.progress = observe_surface(&surface, &self.targets, self.progress)?;
                }
                _ => {}
            }
        }
        Ok(self.progress)
    }

    /// Observe natural completion by the same absolute output deadline.
    pub(in super::super) fn finish(mut self) -> Result<Receipt, Error> {
        while !self.progress.complete {
            self.pump(self.deadline)?;
        }
        Ok(Receipt {
            final_step: self.progress.last_step,
            complete: self.progress.complete,
        })
    }

    fn receive(&self, deadline: Instant) -> Result<ClientEvent, Error> {
        let event = self
            .peer
            .client
            .events
            .recv_deadline(deadline)
            .map_err(|error| {
                if error.is_timeout() {
                    Error::Deadline
                } else {
                    Error::Disconnected
                }
            })?;
        // recv_deadline can return queued events after expiry. This admission check
        // is the documented clock-rule exception; only the receive drives waiting.
        if Instant::now() >= deadline {
            return Err(Error::Deadline);
        }
        match event {
            ClientEvent::CommandRejected { reason, .. } => Err(Error::CommandRejected {
                kind: reason.kind(),
            }),
            ClientEvent::Disconnected { .. } | ClientEvent::VersionMismatch(_) => {
                Err(Error::Disconnected)
            }
            event => Ok(event),
        }
    }

    fn check_snapshot(&self, snapshot: &ClientShellSnapshot) -> Result<(), Error> {
        if snapshot.boot_id != self.targets.boot {
            return Err(Error::BootChanged);
        }
        if snapshot.focused_workspace_id.as_deref() != Some(self.targets.workspace.as_str())
            || snapshot.focused_pane_id.as_deref() != Some(self.targets.pane.as_str())
        {
            return Err(Error::PaneChanged);
        }
        Ok(())
    }
}

fn command() -> String {
    // Neither complete marker is contiguous in the command echo.
    format!(
        "$f='SP_PROFILE_{{0}} {{1}}:{{2}}:{{3:00}}!'; $n={FINAL_STEP}; for ($i=1; $i -le $n; $i++) {{ $f -f 'READY','{OUTPUT_TAG}','STEP',$i; [Threading.Thread]::Sleep({STEP_MS}) }}; $f -f 'READY','{OUTPUT_TAG}','DONE',$n"
    )
}

fn observe_surface(
    surface: &PaneSurfaceFrame,
    targets: &Targets,
    progress: Progress,
) -> Result<Progress, Error> {
    if surface.boot_id != targets.boot {
        return Err(Error::BootChanged);
    }
    let mut panes = surface
        .panes
        .iter()
        .filter(|pane| pane.pane_id == targets.pane);
    let pane = panes.next().ok_or(Error::PaneChanged)?;
    if panes.next().is_some() {
        return Err(Error::PaneChanged);
    }
    observe_frame(&surface.frame, pane.inner_rect, progress)
}

fn observe_frame(
    frame: &FrameData,
    pane: SurfaceRect,
    previous: Progress,
) -> Result<Progress, Error> {
    let width = usize::from(frame.width);
    let x = usize::from(pane.x);
    let right = x + usize::from(pane.width);
    let bottom = usize::from(pane.y) + usize::from(pane.height);
    if pane.width == 0 || pane.height == 0 || right > width || bottom > usize::from(frame.height) {
        return Err(Error::PaneGeometry);
    }
    let mut progress = previous;
    let mut visible_step = None;
    let mut complete = false;
    for row in frame
        .cells
        .chunks_exact(width)
        .take(bottom)
        .skip(usize::from(pane.y))
    {
        let text: String = row[x..right]
            .iter()
            .map(|cell| cell.symbol.as_str())
            .collect();
        match parse_marker(&text)? {
            Some(Marker::Step(step)) => {
                if complete {
                    return Err(Error::Completion);
                }
                if let Some(last) = visible_step {
                    require_step(last + 1, step)?;
                }
                visible_step = Some(step);
                if step > progress.last_step {
                    require_step(progress.last_step + 1, step)?;
                    progress.last_step = step;
                }
            }
            Some(Marker::Done(step)) => {
                if complete
                    || step != FINAL_STEP
                    || progress.last_step != FINAL_STEP
                    || visible_step.is_some_and(|last| last != FINAL_STEP)
                {
                    return Err(Error::Completion);
                }
                complete = true;
            }
            None => {}
        }
    }
    if let Some(observed) = visible_step
        && observed < previous.last_step
    {
        return Err(Error::Regression {
            previous: previous.last_step,
            observed,
        });
    }
    progress.complete |= complete;
    Ok(progress)
}

fn require_step(expected: u8, observed: u8) -> Result<(), Error> {
    if expected != observed {
        return Err(Error::Sequence { expected, observed });
    }
    Ok(())
}

enum Marker {
    Step(u8),
    Done(u8),
}

fn parse_marker(row: &str) -> Result<Option<Marker>, Error> {
    let Some((payload, _)) = row
        .trim_start()
        .strip_prefix(MARKER)
        .and_then(|row| row.strip_prefix(' '))
        .and_then(|row| row.strip_prefix(OUTPUT_TAG))
        .and_then(|row| row.split_once('!'))
    else {
        // A surface can arrive while the console is still writing a row.
        return Ok(None);
    };
    let (kind, number) = payload
        .strip_prefix(':')
        .and_then(|payload| payload.split_once(':'))
        .ok_or(Error::MalformedMarker)?;
    let [tens @ b'0'..=b'9', units @ b'0'..=b'9'] = *number.as_bytes() else {
        return Err(Error::MalformedMarker);
    };
    let step = (tens - b'0') * 10 + (units - b'0');
    if !(1..=FINAL_STEP).contains(&step) {
        return Err(Error::MalformedMarker);
    }
    match kind {
        "STEP" => Ok(Some(Marker::Step(step))),
        "DONE" => Ok(Some(Marker::Done(step))),
        _ => Err(Error::MalformedMarker),
    }
}

#[cfg(test)]
mod tests;
