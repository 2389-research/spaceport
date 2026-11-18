//! Fixtures the crate's tests share.

#![allow(clippy::unwrap_used)]

use crate::Live;
use sheprd_protocol::{
    AgentStatus, ClientShellSnapshot, SemanticNotification, SemanticNotificationKind as Kind,
    SemanticNotificationSound as Sound,
};
use std::{
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

/// One endpoint's live state as a poll reads it, owned by a test so it can
/// change between polls as a reconnection or a newer snapshot would.
pub(crate) struct Endpoint {
    pub(crate) cancel: Arc<AtomicBool>,
    pub(crate) connection_cancel: Arc<AtomicBool>,
    pub(crate) connected: bool,
    pub(crate) reload: bool,
    pub(crate) events: VecDeque<(Instant, SemanticNotification)>,
    pub(crate) snapshot: Option<Arc<ClientShellSnapshot>>,
}

impl Endpoint {
    /// A connected endpoint whose first agent, on the fixture's pane, has
    /// `status`, focused in the fixture's workspace and tab.
    pub(crate) fn new(status: AgentStatus) -> Self {
        let mut snapshot: ClientShellSnapshot = serde_json::from_str(include_str!(
            "../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
        ))
        .unwrap();
        snapshot.agents.truncate(1);
        snapshot.agents[0].pane_id = "pane".into();
        snapshot.agents[0].agent_status = status;
        snapshot.focused_tab_id = Some("tab".into());
        snapshot.focused_workspace_id = Some("workspace".into());
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            connection_cancel: Arc::new(AtomicBool::new(false)),
            connected: true,
            reload: false,
            events: VecDeque::new(),
            snapshot: Some(Arc::new(snapshot)),
        }
    }

    /// What a poll reads of this endpoint, and the queues it drains.
    pub(crate) fn live(&mut self) -> Live<'_> {
        Live {
            cancel: &self.cancel,
            connection_cancel: &self.connection_cancel,
            connected: self.connected,
            reload: &mut self.reload,
            events: &mut self.events,
            snapshot: self.snapshot.as_deref(),
        }
    }
}

/// A notification from the fixture's workspace, tab, and pane.
pub(crate) fn event(kind: Kind) -> SemanticNotification {
    SemanticNotification {
        kind,
        title: "test".into(),
        body: None,
        sound: Some(Sound::Done),
        agent: Some("claude".into()),
        workspace_id: Some("workspace".into()),
        tab_id: Some("tab".into()),
        pane_id: Some("pane".into()),
        position: None,
    }
}
