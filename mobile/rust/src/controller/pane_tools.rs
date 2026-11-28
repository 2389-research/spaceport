// ABOUTME: Exposes the pane trail and the actions this Herdr endpoint advertises.
// ABOUTME: Rechecks the selected pane before queueing an addressed command once.
use super::*;
use spaceport_client::{
    Method, pane_action::Action, pane_history::Step, protocol::ClientShellSnapshot,
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TrailHost {
    hostname: String,
    username: String,
    port: u16,
    fingerprint: String,
    session: String,
}

impl TrailHost {
    pub(crate) fn new(host: &SshHost, session: &str) -> Option<Self> {
        Some(Self {
            hostname: host.hostname.clone(),
            username: host.username.clone(),
            port: host.port,
            fingerprint: host.fingerprint.clone()?,
            session: session.to_owned(),
        })
    }
}

/// An in-memory focus trail. It contains pane IDs and never holds a transport or terminal bytes.
#[derive(uniffi::Object)]
pub struct MobilePaneTrail {
    pub(crate) host: TrailHost,
    pub(crate) history: spaceport_client::pane_history::History,
}

#[derive(Clone, uniffi::Enum)]
pub enum MobilePaneAction {
    SplitRight,
    SplitDown,
    Zoom { enabled: bool },
    Rename { label: String },
}

#[derive(Clone, Copy, Default, uniffi::Record)]
pub struct MobilePaneActions {
    pub split: bool,
    pub zoom: bool,
    pub rename: bool,
}

impl MobilePaneActions {
    pub(crate) fn from_methods(methods: &[String]) -> Self {
        Self {
            split: Method::PaneSplit.advertised_in(methods),
            zoom: Method::PaneZoom.advertised_in(methods),
            rename: Method::PaneRename.advertised_in(methods),
        }
    }

    pub(crate) fn request(
        &self,
        snap: &ClientShellSnapshot,
        boot: &str,
        workspace: &str,
        tab: &str,
        pane: &str,
        action: MobilePaneAction,
    ) -> Result<(Method, serde_json::Value), MobileError> {
        if snap.boot_id != boot
            || !snap.workspaces.iter().any(|w| w.workspace_id == workspace)
            || !snap
                .tabs
                .iter()
                .any(|t| t.tab_id == tab && t.workspace_id == workspace)
            || !snap
                .panes
                .iter()
                .any(|p| p.pane_id == pane && p.tab_id == tab && p.workspace_id == workspace)
        {
            return Err(MobileError::StaleView);
        }
        let request = match &action {
            MobilePaneAction::SplitRight if self.split => Action::SplitRight,
            MobilePaneAction::SplitDown if self.split => Action::SplitDown,
            MobilePaneAction::Zoom { enabled } if self.zoom => Action::SetZoom(*enabled),
            MobilePaneAction::Rename { label } if self.rename => Action::Rename(label),
            _ => return Err(spaceport_client::Error::UnsupportedMethod.into()),
        };
        Ok(request.request(pane))
    }
}

#[uniffi::export]
impl MobileSession {
    pub fn saved_pane_trail(&self) -> Option<Arc<MobilePaneTrail>> {
        lock(&self.projection).save_pane_trail().map(Arc::new)
    }

    pub fn restore_pane_trail(&self, saved: Arc<MobilePaneTrail>) {
        lock(&self.projection).restore_pane_trail(&saved);
    }

    pub fn travel_to_pane(
        &self,
        boot_id: String,
        pane_id: String,
        forward: bool,
    ) -> Result<u64, MobileError> {
        let mut state = self.active(&boot_id)?;
        let revision = state.begin_travel(
            &boot_id,
            &pane_id,
            if forward { Step::Forward } else { Step::Back },
        )?;
        let result = Self::advance_go_to(&self.handle, &mut state);
        self.routes.invalidate(&state);
        result?;
        Ok(revision)
    }

    pub fn apply_pane_action(
        &self,
        boot_id: String,
        workspace_id: String,
        tab_id: String,
        pane_id: String,
        action: MobilePaneAction,
    ) -> Result<(), MobileError> {
        let mut state = self.active(&boot_id)?;
        if state.navigating() || state.go_to_pending() {
            return Err(MobileError::NavigationPending);
        }
        let (method, params) = state.pane_actions.request(
            state.latest_snapshot().ok_or(MobileError::StaleView)?,
            &boot_id,
            &workspace_id,
            &tab_id,
            &pane_id,
            action,
        )?;
        let id = self.handle.request(&boot_id, method, params)?;
        // Even a split of a background pane can change the shared input owner.
        // The ordered barrier identifies the actual result, including a refusal.
        state.start_activation(
            id,
            None,
            None,
            self.handle.set_surface_active(&boot_id, true),
        );
        state.version += 1;
        self.routes.invalidate(&state);
        Ok(())
    }
}
