// ABOUTME: Seeds generated output and controlled status through real private TUI connections.
// ABOUTME: Reads targets from authoritative snapshots and matches every framed response.
use super::report::marker_visible;
use anyhow::{Context as _, Result, bail, ensure};
use serde_json::{Value, json};
use sheprd_client::{
    Client, ClientEvent, ConnectOptions, ConnectTarget, Method,
    protocol::{
        ClientKeyCode, ClientKeyKind, ClientPaneInputEvent, ClientShellSnapshot, PaneSurfaceFrame,
    },
};
use std::{
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

const LIMIT: Duration = Duration::from_secs(20);
pub(super) const GENERATED_LINES: usize = 120;

pub(super) struct Peer {
    client: Client,
    snapshot: Option<Arc<ClientShellSnapshot>>,
    surface: Option<Arc<PaneSurfaceFrame>>,
    snapshots: usize,
    surfaces: usize,
}

pub(super) struct Targets {
    pub(super) boot: String,
    pub(super) workspace: String,
    pub(super) pane: String,
}

impl Peer {
    pub(super) fn open(socket: &Path) -> Result<Self> {
        let client = sheprd_client::connect(
            ConnectTarget::Socket(socket.to_owned()),
            ConnectOptions::default(),
        )?;
        let mut peer = Self {
            client,
            snapshot: None,
            surface: None,
            snapshots: 0,
            surfaces: 0,
        };
        peer.until("initial snapshot", |event| {
            matches!(event, ClientEvent::Snapshot(_))
        })?;
        Ok(peer)
    }

    fn until(&mut self, stage: &str, mut accept: impl FnMut(&ClientEvent) -> bool) -> Result<()> {
        let deadline = Instant::now() + LIMIT;
        loop {
            let event = self
                .client
                .events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .with_context(|| format!("private TUI {stage}; observed={}", self.diagnostic()))?;
            match &event {
                ClientEvent::Snapshot(snapshot) => {
                    self.snapshots += 1;
                    self.snapshot = Some(snapshot.clone());
                }
                ClientEvent::Surface(surface) => {
                    self.surfaces += 1;
                    self.surface = Some(surface.clone());
                }
                ClientEvent::Disconnected { reason, .. } => {
                    bail!("private TUI {stage}: {reason}")
                }
                ClientEvent::CommandRejected { reason, .. } => {
                    bail!("private TUI {stage}: {reason}")
                }
                _ => {}
            }
            if accept(&event) {
                return Ok(());
            }
            ensure!(Instant::now() < deadline, "private TUI {stage} timed out");
        }
    }

    fn request(&mut self, method: Method, params: Value) -> Result<Value> {
        let boot = &self
            .snapshot
            .as_ref()
            .context("missing TUI snapshot")?
            .boot_id;
        let id = self.client.handle.request(boot, method, params)?;
        let mut answer = None;
        self.until("response", |event| {
            if let ClientEvent::Response {
                request_id,
                response,
            } = event
                && *request_id == id
            {
                answer = Some(response.clone());
                return true;
            }
            false
        })?;
        let answer = answer.context("missing correlated TUI response")?;
        ensure!(
            answer.get("error").is_none(),
            "private TUI refused request: {answer}"
        );
        answer
            .get("result")
            .cloned()
            .context("missing TUI response result")
    }

    pub(super) fn seed(&mut self, directory: &Path) -> Result<Targets> {
        let result = self.seed_inner(directory);
        let diagnostic = serde_json::to_vec_pretty(&self.diagnostic())?;
        if let Err(error) = std::fs::write(directory.join("seed-observation.json"), diagnostic) {
            eprintln!("WORKING PROFILE could not retain seed observation: {error}");
        }
        result
    }

    fn diagnostic(&self) -> Value {
        let snapshot = self.snapshot.as_ref().map(|snapshot| json!({
            "boot":snapshot.boot_id,"revision":snapshot.revision,
            "workspace":snapshot.focused_workspace_id,"tab":snapshot.focused_tab_id,"pane":snapshot.focused_pane_id,
            "config_diagnostic":snapshot.config_diagnostic.as_ref().map(|text| text.chars().take(256).collect::<String>())
        }));
        let surface = self.surface.as_ref().map(|surface| {
            let frame = &surface.frame;
            let mut rows: Vec<_> = frame.cells.chunks(usize::from(frame.width).max(1)).rev().take(32)
                .map(|row| row.iter().take(160).flat_map(|cell| cell.symbol.chars()).take(320).collect::<String>()).collect();
            rows.reverse();
            json!({"boot":surface.boot_id,"projection_revision":surface.projection_revision,
                "surface_revision":surface.surface_revision,"width":frame.width,"height":frame.height,
                "panes":surface.panes.iter().take(8).map(|pane| &pane.pane_id).collect::<Vec<_>>(),"last_rows":rows})
        });
        json!({"snapshot_events":self.snapshots,"surface_events":self.surfaces,"snapshot":snapshot,"surface":surface})
    }

    fn seed_inner(&mut self, directory: &Path) -> Result<Targets> {
        let created = self.request(
            Method::WorkspaceCreate,
            json!({"cwd": directory, "focus": true, "label": "Spaceport synthetic profile"}),
        )?;
        let workspace = created["workspace"]["workspace_id"]
            .as_str()
            .context("created workspace ID")?
            .to_owned();
        if self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.focused_workspace_id.as_deref())
            != Some(&workspace)
        {
            self.until("created workspace snapshot", |event| matches!(event, ClientEvent::Snapshot(snapshot) if snapshot.focused_workspace_id.as_deref() == Some(&workspace)))?;
        }
        let snapshot = self
            .snapshot
            .as_ref()
            .context("created workspace snapshot")?;
        let targets = Targets {
            boot: snapshot.boot_id.clone(),
            workspace,
            pane: snapshot
                .focused_pane_id
                .clone()
                .context("created pane ID")?,
        };
        // The input echo contains a format placeholder, never the complete ready marker.
        let command = generated_command();
        self.client.handle.send_input(
            &targets.boot,
            &targets.pane,
            vec![
                ClientPaneInputEvent::TextCommit(command),
                ClientPaneInputEvent::Key {
                    code: ClientKeyCode::Enter,
                    modifiers: 0,
                    kind: ClientKeyKind::Press,
                    repeat_count: 1,
                    shifted_codepoint: None,
                    generated_text: None,
                    tracks_release: false,
                    physical_key_id: None,
                    windows_record: None,
                },
            ],
        )?;
        self.until("generated output marker", |event| matches!(event, ClientEvent::Surface(surface) if marker_visible(&surface.frame)))?;
        Ok(targets)
    }

    pub(super) fn close_workspace(&mut self, workspace: &str) -> Result<()> {
        self.request(
            Method::WorkspaceClose,
            json!({"workspace_id": workspace, "close_group": false}),
        )?;
        Ok(())
    }
}

pub(super) fn generated_command() -> String {
    format!(
        "i=0; while [ \"$i\" -lt {GENERATED_LINES} ]; do printf 'synthetic profile row %03d\\n' \"$i\"; i=$((i+1)); done; printf 'SP_PROFILE_%s\\n' READY"
    )
}

pub(super) fn report_status(api: &Path, pane: &str, state: &str, sequence: u64) -> Result<()> {
    let mut stream = UnixStream::connect(api).context("connect private status API")?;
    stream.set_read_timeout(Some(LIMIT))?;
    stream.set_write_timeout(Some(LIMIT))?;
    let id = format!("synthetic-profile-{sequence}");
    writeln!(
        stream,
        "{}",
        json!({"id": id, "method": "pane.report_agent", "params": {
            "pane_id": pane, "source": "claude-hook", "agent": "claude", "state": state, "seq": sequence
        }})
    )?;
    let mut line = String::new();
    BufReader::new(stream).take(65_537).read_line(&mut line)?;
    ensure!(
        line.len() <= 65_536 && line.ends_with('\n'),
        "invalid private status response"
    );
    let response: Value = serde_json::from_str(&line)?;
    ensure!(
        response["id"] == id && response.get("error").is_none(),
        "private synthetic status refused: {response}"
    );
    Ok(())
}
