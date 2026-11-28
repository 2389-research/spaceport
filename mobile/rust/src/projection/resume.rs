// ABOUTME: Fences a retained SSH session until Herdr acknowledges a fresh request.
// ABOUTME: Prevents old keyboard targets and replies from affecting a new presentation.
use super::Projection;
use crate::MobileError;
use spaceport_activation::TIMEOUT;
use std::time::Instant;

#[derive(Clone)]
pub(super) enum Visibility {
    Active,
    Parked,
    Checking {
        request: String,
        deadline: Instant,
        revision: Option<Result<u64, ()>>,
    },
}

impl Projection {
    pub(crate) fn is_parked(&self) -> bool {
        matches!(self.visibility, Visibility::Parked)
    }

    pub(crate) fn is_active(&self) -> bool {
        matches!(self.visibility, Visibility::Active)
    }

    pub(super) fn accepts_surfaces(&self) -> bool {
        self.release_request.is_none() && !matches!(self.visibility, Visibility::Parked)
    }

    pub(crate) fn park(&mut self) {
        self.reported_focus = false;
        self.surface = None;
        if let Some(latest) = self.pending.take() {
            self.adopt_snapshot(latest);
        }

        self.focus_interrupted = self.focus_interrupted();
        self.cancel_go_to();
        self.visibility = Visibility::Parked;
        self.idle.interrupt();
        self.input_epoch = self.input_epoch.wrapping_add(1);
        self.last_input = None;
        self.reassert_since = None;
        self.version += 1;
    }

    /// Begins proving a retained session. The budget is the one barrier any
    /// activation gets: this is the same acknowledgement, asked again.
    pub(crate) fn start_resume(&mut self, request: String, now: Instant) {
        self.visibility = Visibility::Checking {
            request,
            deadline: now + TIMEOUT,
            revision: None,
        };
    }

    pub(super) fn resume_reply(&mut self, id: &str, accepted_revision: Option<u64>) -> bool {
        if let Visibility::Checking {
            request, revision, ..
        } = &mut self.visibility
            && request == id
        {
            *revision = Some(accepted_revision.ok_or(()));
            return true;
        }
        false
    }

    pub(crate) fn finish_resume(&mut self, id: &str, now: Instant) -> Result<bool, MobileError> {
        let Visibility::Checking {
            request,
            deadline,
            revision,
        } = &self.visibility
        else {
            return Err(MobileError::StaleView);
        };
        if request != id {
            return Err(MobileError::StaleView);
        }
        if matches!(revision, Some(Err(()))) {
            return Err(MobileError::ResumeUnconfirmed);
        }
        if now >= *deadline {
            return Err(if revision.is_none() {
                MobileError::ResumeTimeout
            } else {
                MobileError::ResumeUnconfirmed
            });
        }
        let Some(Ok(minimum)) = revision else {
            return Ok(false);
        };
        let current = self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot.revision >= *minimum
                && (snapshot.focused_pane_id.is_none() || self.coherent().is_some())
        });
        if self.release_request.is_some() || self.navigating() || !current {
            return Ok(false);
        }
        self.visibility = Visibility::Active;
        self.confirm_roster();
        self.focus_interrupted = false;
        self.version += 1;
        Ok(true)
    }

    pub(crate) fn confirm_roster(&mut self) {
        if let Some(snapshot) = &mut self.snapshot {
            snapshot.observed_at_ms = spaceport_client::agent_idle::now_ms();
            self.idle.observe(snapshot, snapshot.observed_at_ms);
        }
        if self.is_parked() {
            self.idle.interrupt();
        }
    }
}
