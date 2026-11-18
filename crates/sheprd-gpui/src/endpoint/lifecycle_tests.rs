// ABOUTME: Exercises GUI lifecycles through isolated sockets and native Windows pipes.
// ABOUTME: Shares endpoint handshakes while keeping platform input fixtures explicit.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::controls::Command;
use gpui::{
    AppContext, InteractiveElement as _, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    point, px, size,
};
#[cfg(unix)]
use gpui::{ClipboardItem, Image, ImageFormat};
use sheprd_client::{
    ClientEvent, Method, SavedHost,
    protocol::{endpoint::*, *},
};
use std::{path::PathBuf, sync::atomic::AtomicU64};

mod transport;
use transport::{Listener, Stream};

mod activation_backpressure;
mod atomic_projection;
mod background_commands;
mod catalog_link;
mod close_pane;
mod close_sequence;
mod core_reconnect_overlay;
mod core_terminal_hosts;
#[cfg(unix)]
mod dead_keys;
mod endpoint_switch;
#[cfg(unix)]
mod file_paste;
mod focus_fences;
mod focus_follows_mouse;
mod focus_follows_mouse_render;
mod focus_pipeline;
#[cfg(unix)]
mod function_keys;
mod group_navigation_choices;
mod group_page_move;
#[cfg(unix)]
mod held_paste;
mod herdr_missing;
mod herdr_stopped;
mod horizontal_wheel;
#[cfg(unix)]
mod image_paste;
#[cfg(unix)]
mod image_paste_native;
mod image_scroll;
mod input_gap;
mod input_settings;
#[cfg(unix)]
mod keyboard;
mod link_clicks;
mod link_stability;
mod mouse_cell_motion;
mod mouse_fence;
mod mouse_frame_gap;
mod mouse_gestures;
mod mouse_targets;
mod notice_fence;
mod popup_projection;
mod prompts;
mod reconnect_backoff;
#[cfg(unix)]
mod release_backpressure;
mod remote_drop;
mod resize_claim_fence;
mod resize_reassert;
mod retry_dialog_fencing;
mod scrollbar_drag;
mod selection_activation_budget;
mod shift_select_hint;
#[cfg(unix)]
mod shifted_keys;
mod smooth_scroll_off;
mod sounds;
mod split_drag;
mod split_equalize;
mod staged_release;
#[cfg(unix)]
mod temporary_drop;
mod toast_handoff;
mod toast_navigation;
mod wake_recovery;
mod wheel_slides;
mod window_notices;
mod workspace_close;
mod workspace_menu_navigation;

struct Server {
    stream: Stream,
    path: PathBuf,
}

// Mount the production keyboard scope without the terminal canvas.
// Lifecycle tests control their own resize acknowledgements.
struct Fixture(gpui::Entity<SheprdWindow>);
impl gpui::Render for Fixture {
    fn render(&mut self, _: &mut gpui::Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.0.update(cx, |view, cx| {
            SheprdWindow::shortcut_scope(cx).track_focus(&view.focus)
        })
    }
}

impl Server {
    /// The next lifecycle frame. Every connection reports the window's theme
    /// once it has a snapshot; `window::tests` covers that, so skip it here.
    fn receive(&mut self) -> ClientMessage {
        loop {
            match read_message(&mut self.stream, MAX_FRAME_SIZE).unwrap() {
                ClientMessage::ClientShellHostTheme { .. } => {}
                message => return message,
            }
        }
    }

    fn respond(&mut self, request: &serde_json::Value) {
        let id = request["id"].as_str().unwrap();
        write_message(
            &mut self.stream,
            &ServerMessage::ClientShellEndpointResponseChunk {
                boot_id: snapshot().boot_id,
                request_id: id.into(),
                final_chunk: true,
                data: serde_json::to_vec(&serde_json::json!({"id": id, "result": {}})).unwrap(),
            },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

fn surface(snapshot: &ClientShellSnapshot) -> Arc<PaneSurfaceFrame> {
    Arc::new(PaneSurfaceFrame {
        boot_id: snapshot.boot_id.clone(),
        projection_revision: snapshot.revision,
        surface_revision: 1,
        frame: FrameData {
            width: 80,
            height: 24,
            cells: vec![],
            cursor: None,
            hyperlinks: vec![],
            graphics: vec![],
        },
        panes: vec![],
        splits: vec![],
        popup: None,
        graphics: Default::default(),
    })
}

/// Catch a stream of updates that never satisfies the expected state.
/// The separate deadline also bounds a reader that cannot make progress.
const WAKES_PER_WAIT: usize = 10_000;

/// Wait for a reader update, then recheck the condition under the caller's bound.
/// Worker flags may change while a fixture holds the inbox and blocks that reader.
fn await_reader(until: Instant, what: &str) {
    assert!(Instant::now() < until, "{what}");
    crate::connection::applied::wait();
}

/// Poll until the inbox has been projected into `live`, then return.
///
/// `ConnectionBridge::take_update` reads the inbox under `try_lock` so the UI
/// thread never blocks on the socket worker: a poll that races the worker
/// projects nothing and the next one delivers it. A test that polls once and
/// asserts is therefore reading whatever `live` happened to hold, which is a
/// revision behind whenever the worker held the lock. Drive polls until the
/// expected state lands instead of assuming a single poll suffices.
fn project_until(
    view: &mut SheprdWindow,
    cx: &mut Context<SheprdWindow>,
    what: &str,
    ready: impl Fn(&SheprdWindow) -> bool,
) {
    let until = Instant::now() + crate::connection::applied::WAIT_TIMEOUT;
    let mut wakes = 0;
    loop {
        view.poll_endpoints(cx);
        if ready(view) {
            return;
        }
        wakes += 1;
        assert!(wakes < WAKES_PER_WAIT, "{what} was never projected");
        await_reader(until, what);
    }
}

/// `project_until` for a wait whose poll reads what a poll returns, such as the
/// bell it takes or the context it needs, rather than a projected field.
fn poll_until(
    view: &mut SheprdWindow,
    cx: &mut Context<SheprdWindow>,
    mut poll: impl FnMut(&mut SheprdWindow, &mut Context<SheprdWindow>) -> bool,
) {
    let until = Instant::now() + crate::connection::applied::WAIT_TIMEOUT;
    let mut wakes = 0;
    loop {
        if poll(view, cx) {
            return;
        }
        wakes += 1;
        assert!(wakes < WAKES_PER_WAIT, "the window never reached the wait");
        await_reader(until, "the window never reached the wait");
    }
}

fn wait_until(mut ready: impl FnMut() -> bool) {
    let until = Instant::now() + crate::connection::applied::WAIT_TIMEOUT;
    let mut wakes = 0;
    while !ready() {
        wakes += 1;
        assert!(wakes < WAKES_PER_WAIT, "the client worker never settled");
        await_reader(until, "the client worker never settled");
    }
}

fn connected_endpoint(id: &str) -> (Endpoint, Server) {
    connected_endpoint_reporting(id, None)
}

fn connected_endpoint_reporting(id: &str, server_version: Option<&str>) -> (Endpoint, Server) {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "hg-{}-{}.sock",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let mut endpoint = Endpoint::new(
        id.into(),
        id.into(),
        ConnectTarget::Socket(path.clone()),
        true,
    );
    let mut server = start_endpoint(&mut endpoint);
    send_welcome(&mut server, server_version);
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.connection.handle.is_some() && endpoint.live.snapshot.is_some()
    });
    assert!(endpoint.live.supports_surface);
    let frame = surface(endpoint.live.snapshot.as_ref().unwrap());
    endpoint
        .connection
        .inbox
        .lock()
        .unwrap()
        .apply(ClientEvent::Surface(frame));
    endpoint.poll(Instant::now());
    (endpoint, server)
}

fn start_endpoint(endpoint: &mut Endpoint) -> Server {
    let ConnectTarget::Socket(path) = &endpoint.connection.target else {
        panic!("a lifecycle fixture must use its private transport");
    };
    let path = path.clone();
    let listener = Listener::bind(&path).unwrap();
    endpoint.connect(ConnectOptions::default(), true);
    // Accept coordinates with the exact worker, without polling a clock.
    Server {
        stream: listener.accept().unwrap(),
        path,
    }
}

fn send_welcome(server: &mut Server, server_version: Option<&str>) {
    assert!(matches!(
        server.receive(),
        ClientMessage::EndpointControl { .. }
    ));
    let mut welcome: EndpointServerWelcome = serde_json::from_str(include_str!(
        "../../../sheprd-protocol/tests/fixtures/endpoint-welcome-v1.json"
    ))
    .unwrap();
    if let Some(version) = server_version {
        welcome.server_version = version.into();
    }
    welcome.capabilities.extend([
        "surface_interest".into(),
        "presentation_effects_fence".into(),
    ]);
    welcome.methods.extend(
        [
            Method::ClientShellSurfaceSet,
            Method::WorkspaceCreate,
            Method::WorkspaceClose,
            Method::TabCreate,
            Method::PaneSplit,
            Method::LayoutSetSplitRatio,
            Method::TabFocus,
            Method::PaneFocus,
            Method::WorkspaceFocus,
            Method::PaneFocusDirection,
            Method::PaneZoom,
            Method::PaneResize,
            Method::PaneSwap,
            Method::PaneClear,
            Method::PaneScroll,
            Method::PaneClose,
            Method::TabClose,
            Method::CommandInvoke,
            Method::WorktreeCreate,
            Method::WorktreeOpen,
            Method::ServerReloadConfig,
        ]
        .map(|method| method.as_str().to_owned()),
    );
    for (kind, data) in [
        (
            ENDPOINT_WELCOME_KIND,
            serde_json::to_string(&welcome).unwrap(),
        ),
        (
            ENDPOINT_SNAPSHOT_KIND,
            serde_json::to_string(&snapshot()).unwrap(),
        ),
    ] {
        write_message(
            &mut server.stream,
            &ServerMessage::EndpointControl {
                kind: kind.into(),
                data,
            },
            MAX_GRAPHICS_FRAME_SIZE,
        )
        .unwrap();
    }
}

fn prepare_mouse(view: &mut SheprdWindow, endpoint: Endpoint, cx: &mut Context<SheprdWindow>) {
    view.endpoints.truncate(1);
    view.endpoints.push(endpoint);
    view.selected_endpoint = 1;
    view.options = ConnectOptions::default();
    view.reset_selected(cx);
    view.live
        .activation
        .transition(crate::state::activation::Transition::Reset(None));
    let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
    let mut inactive = snapshot.panes[0].clone();
    inactive.pane_id = "w1:p2".into();
    inactive.focused = false;
    snapshot.panes.push(inactive);
    Arc::make_mut(view.live.surface.as_mut().unwrap()).panes = [0, 40]
        .into_iter()
        .map(|x| PaneSurfacePane {
            pane_id: if x == 0 { "w1:p1" } else { "w1:p2" }.into(),
            content_revision: 1,
            rect: SurfaceRect {
                x,
                y: 0,
                width: 40,
                height: 24,
            },
            inner_rect: SurfaceRect {
                x: x + 1,
                y: 1,
                width: 38,
                height: 22,
            },
            scrollbar_rect: None,
            scroll: None,
            focused: x == 0,
            mouse_reporting: true,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 760,
            pixel_height: 880,
        })
        .collect();
    view.cell_width = 10.;
    view.bounds = gpui::Bounds::new(
        point(px(100.), px(50.)),
        size(px(800.), px(24. * view.config.terminal.line_height())),
    );
    // The reducer owns the same coherent fixture the window hit tests.
    let mut inbox = view.endpoints[1].connection.inbox.lock().unwrap();
    inbox.snapshot = view.live.snapshot.clone();
    inbox.surface = view.live.surface.clone();
    assert!(view.input_ready());
}

fn mouse_position(view: &SheprdWindow, column: f32, row: f32) -> gpui::Point<gpui::Pixels> {
    view.bounds.origin
        + point(
            px(column * 10.),
            px(row * view.config.terminal.line_height()),
        )
}

fn mouse_event(kind: ClientMouseKind, column: u16, row: u16) -> ClientPaneInputEvent {
    ClientPaneInputEvent::Mouse {
        kind,
        position: ClientMousePosition::Cell { column, row },
        geometry: None,
        modifiers: 0,
        lines: 1,
    }
}

#[cfg(unix)]
fn clipboard_image(bytes: &[u8]) -> ClipboardItem {
    ClipboardItem::new_image(&Image::from_bytes(ImageFormat::Png, bytes.to_vec()))
}

#[cfg(unix)]
fn prepare_remote_image(
    view: &mut SheprdWindow,
    mut endpoint: Endpoint,
    cx: &mut Context<SheprdWindow>,
) {
    // Only change the classification after connecting the isolated socket harness.
    endpoint.connection.target = ConnectTarget::Ssh {
        target: "unused-image-test.invalid".into(),
        session: "default".into(),
    };
    prepare_mouse(view, endpoint, cx);
    assert!(view.accepts_remote_images());
}

#[cfg(unix)]
fn image_popup(view: &mut SheprdWindow, id: &str) {
    let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
    surface.popup = Some(Box::new(ClientShellPopupSurface {
        terminal_id: id.into(),
        title: String::new(),
        width: None,
        height: None,
        frame: surface.frame.clone(),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 800,
        pixel_height: 480,
    }));
}

#[cfg(unix)]
fn wait_image_finished(view: &gpui::Entity<SheprdWindow>, cx: &mut gpui::VisualTestContext) {
    // GPUI tasks publish the frame and the socket worker finishes its FIFO
    // slot: the executor is this thread's to run, and the slot ends on a thread
    // that delivers no inbox update, so neither may wait on the other.
    loop {
        // Every preparation step only this thread can run.
        let (waiting, preparing) = view.update(cx, |view, _| {
            view.cancel_stale_image();
            (
                view.pending_upload_handles(),
                view.pending_images_are_preparing(),
            )
        });
        if waiting.is_empty() && !preparing {
            return;
        }
        if preparing {
            cx.run_until_parked();
            continue;
        }
        // What is left was handed to the socket worker, which ends its slot
        // without an inbox update; block on the slot itself.
        for handle in waiting {
            handle.wait_finished();
        }
    }
}
