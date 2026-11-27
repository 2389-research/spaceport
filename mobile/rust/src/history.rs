// ABOUTME: Searches retained Herdr scrollback and reads the selected match in its original case.
// ABOUTME: Fences every request to the pane and revision that supply its cell coordinates.
use super::{MobileError, MobileSession};
use crate::InputTarget;
use spaceport_client::response::EndpointErrorCode;
use spaceport_client::scrollback::{
    CopyMotion, CopyMotionParams, CopySearchParams, ScrollbackResponse, SearchDirection,
    SelectionReadParams, TextPoint, TextRange, content_settled, retryable_after,
};
use std::{
    collections::HashMap,
    sync::{Arc, PoisonError, mpsc},
    time::{Duration, Instant},
};

/// How long a search waits for the pane's content to settle before it gives up
/// and asks the user to look at the current view.
const SETTLE: Duration = Duration::from_secs(5);

type Reply = mpsc::Sender<spaceport_client::Result<ScrollbackResponse>>;

#[derive(Clone, Default)]
pub(crate) struct HistoryInbox {
    generation: u64,
    // Canceled requests keep only their ID until the ordered worker delivers the reply.
    replies: HashMap<String, Option<Reply>>,
}

impl HistoryInbox {
    pub(crate) fn take_reply(&mut self, id: &str) -> Option<Option<Reply>> {
        self.replies.remove(id)
    }

    fn begin(&mut self) -> Result<u64, MobileError> {
        // Bound even a client submitting new queries faster than the host can acknowledge them.
        if self.replies.len() >= 8 {
            return Err(MobileError::HistoryBusy);
        }
        self.cancel();
        Ok(self.generation)
    }

    fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        for reply in self.replies.values_mut() {
            *reply = None;
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("The terminal changed during the search. Tap Find to search again.")]
    Changed,
    #[error(transparent)]
    Mobile(#[from] MobileError),
}

impl From<spaceport_client::Error> for Error {
    fn from(error: spaceport_client::Error) -> Self {
        match error {
            spaceport_client::Error::Endpoint {
                code: EndpointErrorCode::StaleContent,
                ..
            } => Self::Changed,
            other => Self::Mobile(other.into()),
        }
    }
}

impl From<Error> for MobileError {
    fn from(error: Error) -> Self {
        match error {
            Error::Changed => Self::Client {
                message: error.to_string(),
            },
            Error::Mobile(error) => error,
        }
    }
}

#[derive(Clone, uniffi::Record)]
pub struct HistoryCursor {
    target: InputTarget,
    revision: u64,
    query: String,
    start_row: u32,
    start_col: u16,
    end_row: u32,
    end_col: u16,
}

#[derive(uniffi::Record)]
pub struct HistoryMatch {
    pub cursor: HistoryCursor,
    pub index: u64,
    pub total: u64,
    pub before: String,
    pub matched: String,
    pub after: String,
}

#[uniffi::export]
impl MobileSession {
    pub fn history_search(
        self: &Arc<Self>,
        target: InputTarget,
    ) -> Result<Arc<HistorySearch>, MobileError> {
        let mut state = self.current(&target.boot_id)?;
        state.history_pane(&target)?;
        let generation = state.history.begin()?;
        Ok(Arc::new(HistorySearch {
            session: self.clone(),
            target,
            generation,
        }))
    }
}

impl MobileSession {
    /// The pane's content revision this search may name, waiting for the next
    /// frame while the pane is mid-write or still shows a revision the daemon
    /// already refused. The desktop window refuses the same two cases before
    /// sending; the daemon answers `stale_content` for either.
    ///
    /// Blocking: run on a background thread. The drain signals `frames` for
    /// every event it takes in, so this waits on a frame rather than polling.
    fn settled_revision(
        &self,
        target: &InputTarget,
        refused: Option<u64>,
    ) -> Result<u64, MobileError> {
        let started = Instant::now();
        let mut state = self.current(&target.boot_id)?;
        loop {
            if state.disconnected || self.handle.is_disconnected() {
                return Err(MobileError::Disconnected);
            }
            let revision = state.history_pane(target)?.content_revision;
            let ready = match refused {
                Some(refused) => retryable_after(refused, revision),
                None => content_settled(revision),
            };
            if ready {
                return Ok(revision);
            }
            let waited = SETTLE.saturating_sub(started.elapsed());
            if waited.is_zero() {
                // Nothing settled: the phone's view keeps changing under it.
                return Err(MobileError::StaleView);
            }
            let (released, _) = self
                .frames
                .wait_timeout(state, waited)
                .unwrap_or_else(PoisonError::into_inner);
            state = released;
        }
    }
}

#[derive(uniffi::Object)]
pub struct HistorySearch {
    session: Arc<MobileSession>,
    target: InputTarget,
    generation: u64,
}

#[uniffi::export]
impl HistorySearch {
    pub fn cancel(&self) {
        let mut state = super::lock(&self.session.projection);
        if state.history.generation == self.generation {
            state.history.cancel();
        }
    }

    /// Starts at the newest match and wraps at either end when stepping.
    pub fn find(
        &self,
        query: String,
        newer: bool,
        previous: Option<HistoryCursor>,
    ) -> Result<Option<HistoryMatch>, MobileError> {
        if query.is_empty() {
            return Ok(None);
        }
        let revision = self.session.settled_revision(&self.target, None)?;
        match self.find_at(&query, newer, previous.as_ref(), revision) {
            Err(Error::Changed) => {
                // The daemon moved on between the frame read here and the
                // request: wait for a frame the daemon has moved on to, then ask
                // once more, as the window does.
                let next = self
                    .session
                    .settled_revision(&self.target, Some(revision))?;
                self.find_at(&query, newer, None, next).map_err(Into::into)
            }
            result => result.map_err(Into::into),
        }
    }
}

impl HistorySearch {
    fn find_at(
        &self,
        query: &str,
        newer: bool,
        previous: Option<&HistoryCursor>,
        revision: u64,
    ) -> Result<Option<HistoryMatch>, Error> {
        let target = &self.target;
        let previous =
            previous.filter(|p| &p.target == target && p.query == query && p.revision == revision);
        let params = CopySearchParams {
            pane_id: target.pane_id.clone(),
            query: query.to_owned(),
            direction: if newer {
                SearchDirection::Forward
            } else {
                SearchDirection::Backward
            },
            cursor: TextPoint {
                row: u32::MAX,
                col: u16::MAX,
            },
            content_revision: revision,
            previous: previous.map(|p| TextRange {
                start: TextPoint {
                    row: p.start_row,
                    col: p.start_col,
                },
                end: TextPoint {
                    row: p.end_row,
                    col: p.end_col,
                },
            }),
        };
        let ScrollbackResponse::PaneCopySearch(found) =
            self.request(|| self.session.handle.copy_search(&target.boot_id, &params))?
        else {
            return Err(MobileError::HistoryResult.into());
        };
        if found.pane_id != target.pane_id || found.content_revision != revision {
            return Err(MobileError::HistoryResult.into());
        }
        if found.total == 0 {
            return Ok(None);
        }
        let range = found
            .current_match()
            .filter(|r| r.start <= r.end)
            .ok_or(MobileError::HistoryResult)?;
        let index = found
            .current_global
            .filter(|i| *i < found.total)
            .ok_or(MobileError::HistoryResult)?
            + 1;
        let read = |anchor, cursor| -> Result<String, Error> {
            let params = SelectionReadParams {
                pane_id: target.pane_id.clone(),
                anchor,
                cursor,
                content_revision: Some(revision),
            };
            match self.request(|| self.session.handle.read_selection(&target.boot_id, &params))? {
                ScrollbackResponse::PaneSelection(s) if s.pane_id == target.pane_id => Ok(s.text),
                _ => Err(MobileError::HistoryResult.into()),
            }
        };
        let matched = read(range.start, range.end)?;
        // Include the match in the bordering reads. Herdr trims trailing blank cells.
        // It also owns the cell-to-text mapping for wide and combining characters.
        let before = if range.start.col > 0 {
            let first = TextPoint {
                col: range.start.col.saturating_sub(64),
                ..range.start
            };
            let text = read(first, range.end)?;
            text.strip_suffix(&matched)
                .ok_or(MobileError::HistoryResult)?
                .to_owned()
        } else {
            String::new()
        };
        // The SSH PTY may be narrower than the endpoint surface. Ask Herdr for the
        // actual row end before reading right-hand context.
        let motion = CopyMotionParams {
            pane_id: target.pane_id.clone(),
            cursor: range.end,
            motion: CopyMotion::LineEnd,
            content_revision: Some(revision),
        };
        let ScrollbackResponse::PaneCopyMotion(line) =
            self.request(|| self.session.handle.copy_motion(&target.boot_id, &motion))?
        else {
            return Err(MobileError::HistoryResult.into());
        };
        if line.pane_id != target.pane_id
            || line.content_revision != revision
            || line.cursor.row != range.end.row
        {
            return Err(MobileError::HistoryResult.into());
        }
        let last_col = range.end.col.saturating_add(64).min(line.cursor.col);
        let after = if last_col > range.end.col {
            let last = TextPoint {
                col: last_col,
                ..range.end
            };
            let text = read(range.start, last)?;
            text.strip_prefix(&matched)
                .ok_or(MobileError::HistoryResult)?
                .to_owned()
        } else {
            String::new()
        };
        Ok(Some(HistoryMatch {
            cursor: HistoryCursor {
                target: target.clone(),
                revision,
                query: params.query,
                start_row: range.start.row,
                start_col: range.start.col,
                end_row: range.end.row,
                end_col: range.end.col,
            },
            index,
            total: found.total,
            before,
            matched,
            after,
        }))
    }

    fn request(
        &self,
        send: impl FnOnce() -> spaceport_client::Result<String>,
    ) -> Result<ScrollbackResponse, Error> {
        let target = &self.target;
        let (tx, rx) = mpsc::channel();
        let id = {
            let mut state = self.session.current(&target.boot_id)?;
            state.history_pane(target)?;
            if state.history.generation != self.generation {
                return Err(MobileError::StaleView.into());
            }
            // The drain takes this lock before delivering replies, including immediate rejections.
            let id = send()?;
            state.history.replies.insert(id.clone(), Some(tx));
            id
        };
        let reply = rx.recv_timeout(Duration::from_secs(5));
        let mut state = self.session.current(&target.boot_id)?;
        if reply.is_err()
            && let Some(sender) = state.history.replies.get_mut(&id)
        {
            *sender = None;
        }
        state.history_pane(target)?;
        if state.history.generation != self.generation {
            return Err(MobileError::StaleView.into());
        }
        reply
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => MobileError::HistoryTimeout,
                mpsc::RecvTimeoutError::Disconnected => MobileError::Disconnected,
            })?
            .map_err(Into::into)
    }
}
