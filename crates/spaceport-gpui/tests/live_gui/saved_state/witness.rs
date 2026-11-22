// ABOUTME: Observes the pinned daemon's handshake, workspace and shutdown on its private socket.
// ABOUTME: Drains bounded wire frames until EOF without requesting terminal surfaces.
use super::*;
use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};
use spaceport_client::{
    ConnectOptions, Stream,
    protocol::{
        ClientMessage, ClientShellSnapshot, MAX_FRAME_SIZE, ServerMessage, endpoint::*,
        read_message, write_message,
    },
};
use std::{
    io::{BufRead, BufReader},
    net::Shutdown,
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};

#[derive(Debug)]
enum Observation {
    Ready {
        version: String,
        boot: String,
        workspace: String,
    },
    Shutdown(Option<String>),
    Eof,
}

// Both the startup and shutdown waits hold this guard. Closing the socket
// releases a blocked reader even if the scenario fails between GUI launches.
struct Observer {
    stream: Stream,
    worker: Option<thread::JoinHandle<()>>,
    events: Receiver<Observation>,
}

impl Drop for Observer {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Observer {
    fn next(&self, deadline: Instant) -> Observation {
        self.events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("private Herdr observer failed or exceeded its deadline")
    }
}

fn observe(stream: Stream, events: SyncSender<Observation>) {
    let mut wire = BufReader::new(stream);
    let message = read_message(&mut wire, MAX_FRAME_SIZE).unwrap();
    let ServerMessage::EndpointControl { kind, data } = message else {
        panic!("expected the endpoint welcome")
    };
    assert_eq!(kind, ENDPOINT_WELCOME_KIND);
    let welcome: EndpointServerWelcome = serde_json::from_str(&data).unwrap();
    assert_eq!(welcome.generation, ENDPOINT_PROTOCOL_GENERATION);
    assert!(
        welcome.error.is_none(),
        "welcome rejected: {:?}",
        welcome.error
    );
    assert_eq!(welcome.snapshot_codec, SNAPSHOT_CODEC_V1);
    assert_eq!(welcome.surface_codec, SURFACE_CODEC_V1);
    assert_eq!(welcome.input_codec, INPUT_CODEC_V1);
    assert_eq!(welcome.blob_codec, BLOB_CODEC_V1);
    assert!(
        welcome
            .capabilities
            .iter()
            .any(|cap| cap == "surface_interest")
    );
    let mut version = Some(welcome.server_version);
    let mut reported_shutdown = false;
    // Empty input at a frame boundary is EOF. A truncated frame fails read_message.
    while !wire.fill_buf().unwrap().is_empty() {
        match read_message(&mut wire, MAX_FRAME_SIZE).unwrap() {
            ServerMessage::EndpointControl { kind, data }
                if version.is_some() && kind == ENDPOINT_SNAPSHOT_KIND =>
            {
                let snapshot: ClientShellSnapshot = serde_json::from_str(&data).unwrap();
                if let Some(workspace) = snapshot.focused_workspace_id {
                    assert!(!workspace.is_empty() && !snapshot.boot_id.is_empty());
                    events
                        .send(Observation::Ready {
                            version: version.take().unwrap(),
                            boot: snapshot.boot_id,
                            workspace,
                        })
                        .unwrap();
                }
            }
            ServerMessage::ServerShutdown { reason } => {
                assert_eq!(reason.as_deref(), Some("server is shutting down"));
                // Herdr announces the same stop during initiation and completion.
                if !reported_shutdown {
                    reported_shutdown = true;
                    events.send(Observation::Shutdown(reason)).unwrap();
                }
            }
            _ => {}
        }
    }
    events.send(Observation::Eof).unwrap();
}

pub(super) struct Witness {
    observer: Observer,
    version: String,
    server_pid: u32,
    boot: String,
    pub(super) workspace: String,
}

impl Witness {
    pub(super) fn open(socket: PathBuf, server_pid: u32) -> Self {
        let deadline = Instant::now() + TIMEOUT;
        let mut stream = Stream::connect(socket).unwrap();
        let peer = getsockopt(&stream, PeerCredentials).unwrap();
        assert_eq!(u32::try_from(peer.pid()).unwrap(), server_pid);
        stream.set_write_timeout(Some(TIMEOUT)).unwrap();
        let options = ConnectOptions::default();
        let hello = EndpointClientHello {
            generation: ENDPOINT_PROTOCOL_GENERATION,
            cell_width_px: options.cell_width_px,
            cell_height_px: options.cell_height_px,
            surface_size: options.surface_size,
            pixel_mouse: false,
            direct_graphics: false,
            endpoint_keybindings: false,
            mouse_capture: false,
            surface_active: false,
            surface_reuse: false,
            surface_delta: false,
            surface_scroll: false,
            snapshot_codecs: vec![SNAPSHOT_CODEC_V1.into()],
            surface_codecs: vec![SURFACE_CODEC_V1.into()],
            input_codecs: vec![INPUT_CODEC_V1.into()],
            blob_codecs: vec![BLOB_CODEC_V1.into()],
        };
        write_message(
            &mut stream,
            &ClientMessage::EndpointControl {
                kind: ENDPOINT_HELLO_KIND.into(),
                data: serde_json::to_string(&hello).unwrap(),
            },
            MAX_FRAME_SIZE,
        )
        .unwrap();
        let reader = stream.try_clone().unwrap();
        // The observer emits at most one ready receipt, one shutdown and one EOF.
        let (send, events) = sync_channel(3);
        let observer = Observer {
            stream,
            worker: Some(thread::spawn(move || observe(reader, send))),
            events,
        };
        let first = observer.next(deadline);
        let Observation::Ready {
            version,
            boot,
            workspace,
        } = first
        else {
            panic!("expected a welcome and focused workspace before {first:?}")
        };
        Self {
            observer,
            version,
            server_pid,
            boot,
            workspace,
        }
    }

    pub(super) fn receipt(&self) -> String {
        serde_json::json!({"pid": self.server_pid, "generation": ENDPOINT_PROTOCOL_GENERATION, "server_version": self.version, "boot": self.boot, "focused_workspace": self.workspace}).to_string()
    }

    pub(super) fn shutdown(&self, receipt: &Path) {
        let mut log = File::create(receipt).unwrap();
        let deadline = Instant::now() + TIMEOUT;
        let first = self.observer.next(deadline);
        let Observation::Shutdown(reason) = first else {
            writeln!(log, "{first:?}").unwrap();
            panic!("expected typed ServerShutdown before {first:?}");
        };
        writeln!(log, "ServerShutdown {reason:?}").unwrap();
        let end = self.observer.next(deadline);
        writeln!(log, "{end:?}").unwrap();
        assert!(
            matches!(end, Observation::Eof),
            "expected EOF after ServerShutdown"
        );
    }
}
