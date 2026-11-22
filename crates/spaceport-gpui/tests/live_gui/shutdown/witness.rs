// ABOUTME: Observes handshake, focused workspace and ordered shutdown events on the real private socket.
// ABOUTME: Uses inactive surface interest so the witness does not drive terminal presentation.
use super::*;
use spaceport_client::{
    Client, ClientEvent, ConnectTarget, connect_with_surface_active, protocol::ServerMessage,
};

pub(super) struct Witness {
    client: Client,
    version: String,
    server_pid: u32,
    pub(super) boot: String,
    pub(super) workspace: String,
}

fn event(client: &Client, deadline: Instant) -> ClientEvent {
    assert!(
        Instant::now() < deadline,
        "timed out waiting for Herdr protocol event"
    );
    client
        .events
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .unwrap()
}

impl Witness {
    pub(super) fn open(socket: PathBuf, server_pid: u32) -> Self {
        let client =
            connect_with_surface_active(ConnectTarget::Socket(socket), Default::default(), false)
                .unwrap();
        let deadline = Instant::now() + TIMEOUT;
        let first = event(&client, deadline);
        let ClientEvent::Connected(welcome) = first else {
            panic!("expected an endpoint welcome, received {first:?}");
        };
        assert_eq!(welcome.generation, 1);
        loop {
            match event(&client, deadline) {
                ClientEvent::Snapshot(snapshot) => {
                    let Some(workspace) = snapshot.focused_workspace_id.clone() else {
                        continue;
                    };
                    assert!(!snapshot.boot_id.is_empty());
                    assert!(!workspace.is_empty());
                    return Self {
                        client,
                        version: welcome.server_version,
                        server_pid,
                        boot: snapshot.boot_id.clone(),
                        workspace,
                    };
                }
                ClientEvent::Disconnected { reason, .. } => {
                    panic!("disconnected before snapshot: {reason}")
                }
                _ => {}
            }
        }
    }

    pub(super) fn receipt(&self) -> String {
        serde_json::json!({"pid": self.server_pid, "generation": 1, "server_version": self.version, "boot": self.boot, "focused_workspace": self.workspace}).to_string()
    }

    pub(super) fn shutdown(&self, receipt: &Path) {
        let mut log = File::create(receipt).unwrap();
        const REASON: &str = "server is shutting down";
        let deadline = Instant::now() + TIMEOUT;
        let mut received = false;
        loop {
            match event(&self.client, deadline) {
                ClientEvent::Message(ServerMessage::ServerShutdown { reason }) => {
                    writeln!(log, "ServerShutdown {reason:?}").unwrap();
                    assert_eq!(reason.as_deref(), Some(REASON));
                    received = true;
                }
                ClientEvent::Disconnected { reason, .. } => {
                    writeln!(log, "Disconnected {reason}").unwrap();
                    assert!(received, "transport closed before typed shutdown");
                    assert_eq!(reason, REASON);
                    return;
                }
                _ => {}
            }
        }
    }
}
