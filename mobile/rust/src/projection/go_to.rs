// ABOUTME: Walks a selected pane through its Herdr workspace and tab on the phone.
// ABOUTME: Keeps the projection's side of the walk; which level comes next is shared.
use super::{OwnedNavigationTarget, Projection};
use crate::MobileError;
use spaceport_activation::{Route, Step};
use std::time::Instant;

/// One Go To request: the projection version it belongs to, and the walk the
/// shared crate sequences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Walk {
    pub(super) request_version: u64,
    pub(super) route: Route,
}

impl Projection {
    pub(crate) fn begin_go_to(
        &mut self,
        boot: &str,
        workspace: String,
        tab: String,
        pane: String,
        expected_agent: Option<String>,
    ) -> Result<u64, MobileError> {
        if self.navigating() || self.route.is_some() {
            return Err(MobileError::NavigationPending);
        }
        let now = Instant::now();
        let request_version = self.version + 1;
        let walk = Walk {
            request_version,
            route: Route::new(workspace, tab, pane, expected_agent, now),
        };
        if self
            .pending
            .as_deref()
            .or(self.snapshot.as_deref())
            .is_none_or(|s| s.boot_id != boot || !walk.route.present(s))
        {
            return Err(MobileError::StaleView);
        }
        self.note_input(now);
        self.route = Some(walk);
        self.go_to_arrival = None;
        self.version = request_version;
        Ok(request_version)
    }

    pub(crate) fn cancel_go_to(&mut self) {
        // A focus request already queued still owns its fence.
        if self.route.take().is_some() {
            self.pane_trail.observe(None, false);
            self.observe_pane();
            self.version += 1;
        }
    }

    pub(crate) fn go_to_pending(&self) -> bool {
        self.route.is_some()
    }

    pub(crate) fn go_to_step(&mut self) -> Result<Option<OwnedNavigationTarget>, MobileError> {
        let Some(walk) = &self.route else {
            return Ok(None);
        };
        let now = Instant::now();
        let latest = self.pending.as_deref().or(self.snapshot.as_deref());
        if self.disconnected
            || latest.is_none_or(|snapshot| !walk.route.present(snapshot))
            || walk.route.expired(now)
        {
            return Err(MobileError::StaleView);
        }
        if self.navigating() || self.pending.is_some() {
            return Ok(None);
        }
        // A step is issued against the projection both halves agree on, so the
        // snapshot is cloned out before the walk records issuing it.
        if self.coherent().is_none() {
            return Ok(None);
        }
        let Some(snapshot) = self.pending.clone().or_else(|| self.snapshot.clone()) else {
            return Ok(None);
        };
        let arrived = self.focused_target().is_some()
            || (self.direct_attach && self.coherent_target().is_some_and(|t| t.popup_id.is_none()));
        let Some(walk) = self.route.as_mut() else {
            return Ok(None);
        };
        let request_version = walk.request_version;
        match walk.route.step(&snapshot) {
            Step::Stale => Err(MobileError::StaleView),
            Step::Focus(step) => Ok(Some(step)),
            Step::Arrived if arrived => {
                self.go_to_arrival = Some(request_version);
                self.route = None;
                self.observe_pane();
                self.version += 1;
                Ok(None)
            }
            Step::Arrived => Ok(None),
        }
    }
}
