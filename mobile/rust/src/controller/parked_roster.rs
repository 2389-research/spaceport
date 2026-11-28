// ABOUTME: Confirms Home's agent roster over a parked terminal's existing SSH bridge.
// ABOUTME: Waits for the daemon's inactive surface receipt and the snapshot revision it names.
use super::{MobileError, MobileSession, MobileView, lock};
use std::{sync::PoisonError, time::Instant};

#[derive(Clone)]
pub(crate) struct ParkedRosterCheck {
    id: String,
    revision: Option<Result<u64, ()>>,
    cancelled: bool,
}

impl ParkedRosterCheck {
    pub(crate) fn reply(&mut self, id: &str, revision: Option<u64>) -> bool {
        if self.id != id {
            return false;
        }
        self.revision = Some(revision.ok_or(()));
        true
    }
}

#[uniffi::export]
impl MobileSession {
    pub fn begin_parked_roster(&self, boot_id: String) -> Result<String, MobileError> {
        let mut state = self.current(&boot_id)?;
        if !state.is_parked() {
            return Err(MobileError::StaleView);
        }
        // The ordered connection owns an outstanding receipt even after its Home view departs.
        if state
            .parked_roster
            .as_ref()
            .is_some_and(|c| c.revision.is_none())
        {
            return Err(MobileError::ResumeUnconfirmed);
        }
        let id = self.handle.set_surface_active(&boot_id, false)?;
        state.parked_roster = Some(ParkedRosterCheck {
            id: id.clone(),
            revision: None,
            cancelled: false,
        });
        Ok(id)
    }

    pub fn cancel_parked_roster(&self, request: String) {
        let mut state = lock(&self.projection);
        if let Some(check) = &mut state.parked_roster
            && check.id == request
        {
            check.cancelled = true;
            self.frames.notify_all();
        }
    }

    /// Blocking. The state drain wakes this observer on each incoming event.
    pub fn wait_parked_roster(
        &self,
        boot_id: String,
        request: String,
    ) -> Result<MobileView, MobileError> {
        let start = Instant::now();
        let mut state = self.current(&boot_id)?;
        loop {
            if state.disconnected || self.handle.is_disconnected() {
                return Err(MobileError::Disconnected);
            }
            let snapshot = state
                .snapshot
                .as_ref()
                .filter(|s| s.boot_id == boot_id)
                .ok_or(MobileError::StaleView)?;
            let check = state
                .parked_roster
                .as_ref()
                .filter(|c| c.id == request && !c.cancelled)
                .ok_or(MobileError::StaleView)?;
            if !state.is_parked() {
                return Err(MobileError::StaleView);
            }
            if let Some(revision) = check.revision {
                let revision = revision.map_err(|()| MobileError::ResumeUnconfirmed)?;
                if snapshot.revision >= revision && state.release_request.is_none() {
                    state.parked_roster = None;
                    state.confirm_roster();
                    return Ok(state.view());
                }
            }
            let remaining = spaceport_activation::TIMEOUT.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(MobileError::RosterTimeout);
            }
            (state, _) = self
                .frames
                .wait_timeout(state, remaining)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}
