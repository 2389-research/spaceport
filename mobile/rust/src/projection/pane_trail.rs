// ABOUTME: Walks the desktop focus trail using mobile's ordered Go To route.
// ABOUTME: Records confirmed arrivals and leaves intermediate route steps out of the trail.
use super::*;
use crate::MobileError;

impl Projection {
    pub(crate) fn save_pane_trail(&self) -> Option<crate::controller::pane_tools::MobilePaneTrail> {
        let host = self.trail_host.clone()?;
        if self.pane_trail.is_empty() {
            return None;
        }
        let mut history = self.pane_trail.clone();
        history.observe(None, false);
        Some(crate::controller::pane_tools::MobilePaneTrail { host, history })
    }

    pub(crate) fn restore_pane_trail(
        &mut self,
        saved: &crate::controller::pane_tools::MobilePaneTrail,
    ) {
        if self.trail_host.as_ref() == Some(&saved.host) {
            self.pane_trail = saved.history.clone();
            self.observe_pane();
            self.version += 1;
        }
    }

    pub(super) fn observe_pane(&mut self) {
        if !self.is_active()
            || self.navigating()
            || self.go_to_pending()
            || self.focused_pane_input().is_none()
        {
            return;
        }
        if let Some(snapshot) = &self.snapshot {
            self.pane_trail.observe(Some(snapshot), false);
        }
    }

    fn focused_pane_input(&self) -> Option<InputTarget> {
        if self.direct_attach {
            self.direct_target()
        } else {
            self.target().filter(|t| t.popup_id.is_none())
        }
    }

    pub(super) fn trail_destination(&self, step: Step) -> Option<String> {
        self.focused_pane_input()?;
        let (.., pane) = self.pane_trail.peek(step, self.snapshot.as_ref()?)?;
        Some(pane.to_owned())
    }

    pub(crate) fn begin_travel(
        &mut self,
        boot: &str,
        expected_pane: &str,
        step: Step,
    ) -> Result<u64, MobileError> {
        let snap = self
            .pending
            .as_ref()
            .or(self.snapshot.as_ref())
            .ok_or(MobileError::StaleView)?;
        let (index, pane) = self
            .pane_trail
            .peek(step, snap)
            .filter(|(_, pane)| *pane == expected_pane)
            .ok_or(MobileError::StaleView)?;
        let target = snap
            .panes
            .iter()
            .find(|p| p.pane_id == pane)
            .ok_or(MobileError::StaleView)?;
        let revision = self.begin_go_to(
            boot,
            target.workspace_id.clone(),
            target.tab_id.clone(),
            target.pane_id.clone(),
            None,
        )?;
        self.pane_trail.begin(index);
        Ok(revision)
    }

    pub(crate) fn latest_snapshot(&self) -> Option<&ClientShellSnapshot> {
        self.pending
            .as_ref()
            .or(self.snapshot.as_ref())
            .map(|s| &*s.snapshot)
    }
}
