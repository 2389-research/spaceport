// ABOUTME: Runs the live-daemon native fixture: actions, keys, text and reconnect.
// ABOUTME: Observes the GUI through its own state, so the normal consumer is what is tested.
use super::*;
use crate::{Command, NavigationTarget, RunCommand, SpaceportWindow};
use anyhow::{Context as _, Result, anyhow, bail};
use gpui::*;
use spaceport_client::{ConnectOptions, ConnectTarget, Method, protocol::*};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[path = "smoke_live/second_window.rs"]
mod second_window;

const STEPS: &[&str] = &[
    "initial painted surface",
    "Cmd-T new tab",
    "Cmd-D right split",
    "Cmd-Shift-D below split",
    "previous tab",
    "next tab",
    "Cmd-Shift-N workspace",
    "workspace navigation",
    "return to full-width tab",
    "text commit + Enter output",
    "native resize",
    "reconnect persisted state",
    "input after reconnect",
    "external workspace pushed to idle GUI",
];

const EXTERNAL_TIMEOUT: Duration = Duration::from_secs(3);

struct ExternalWorkspace {
    // Keep the independent connection alive until the GUI has observed the change.
    _client: spaceport_client::Client,
    id: String,
    sent: Instant,
    responded: Instant,
}

fn create_external_workspace(
    target: ConnectTarget,
    options: ConnectOptions,
    boot: String,
) -> Result<ExternalWorkspace> {
    use spaceport_client::ClientEvent;
    let client = spaceport_client::connect(target, options)
        .context("connecting external workspace client")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut request = None;
    loop {
        let event = client
            .events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .context("external workspace request")?;
        match event {
            ClientEvent::Snapshot(snapshot) if request.is_none() => {
                if snapshot.boot_id != boot {
                    bail!("external client connected to a different daemon boot");
                }
                let sent = Instant::now();
                let id = client
                    .handle
                    .request(
                        &boot,
                        Method::WorkspaceCreate,
                        serde_json::json!({
                            "focus": false, "label": "external-gui-smoke"
                        }),
                    )
                    .context("queueing external workspace request")?;
                request = Some((id, sent));
            }
            ClientEvent::Response {
                request_id,
                response,
            } => {
                let responded = Instant::now();
                let (expected, sent) = request.as_ref().context("unsolicited external response")?;
                if &request_id != expected || spaceport_client::response::result(&response).is_err()
                {
                    bail!("external workspace response: {response}");
                }
                let id = response["result"]["workspace"]["workspace_id"]
                    .as_str()
                    .filter(|id| !id.is_empty())
                    .context("external response missing workspace ID")?
                    .to_owned();
                return Ok(ExternalWorkspace {
                    id,
                    sent: *sent,
                    responded,
                    _client: client,
                });
            }
            ClientEvent::Disconnected { reason, .. } => {
                bail!(reason);
            }
            ClientEvent::CommandRejected { reason, .. } => {
                bail!(reason);
            }
            _ => {}
        }
        if Instant::now() >= deadline {
            bail!("external workspace request timed out");
        }
    }
}

/// Entry: the live daemon fixture behind `--integration-test`.
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 266 lines")]
pub(crate) fn start(handle: WindowHandle<SpaceportWindow>, cx: &mut App) {
    let timer = cx.background_executor().clone();
    cx.spawn(async move |cx| {
        let mut step = 0;
        let mut since = Instant::now();
        let mut boot = String::new();
        let mut workspace = String::new();
        let mut first_tab = String::new();
        let mut second_tab = String::new();
        let mut split_pane = String::new();
        let mut old_size = ClientSurfaceSize { cols: 0, rows: 0 };
        let marker = format!("HERDR_GUI_{}_OK", std::process::id());
        let reconnected_marker = format!("{marker}_RECONNECTED");
        let mut frames = 0_u64;
        let mut external_rx = None;
        let mut external = None;
        let mut baseline = None;
        let mut switched = None;
        let mut completed = false;
        loop {
            timer.timer(Duration::from_millis(100)).await;
            let result = AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<bool> {
                let view = root.downcast::<SpaceportWindow>().map_err(|_| anyhow!("unexpected window root"))?;
                // Observe only: no focus, draw, refresh, request, or reconnect can help
                // deliver this snapshot. The normal GUI event consumer must do it.
                if step == 13 {
                    let view = view.read(cx);
                    let (before, inbox) = baseline.as_ref().context("missing external baseline")?;
                    if !Arc::ptr_eq(inbox, &view.endpoints[view.selected_endpoint].connection.inbox) || !view.live.status.is_connected()
                        || view.endpoints[view.selected_endpoint].connection.handle.as_ref().is_none_or(|h| h.is_disconnected())
                        || view.notices.is_some() || view.live.notices.is_some() {
                        bail!("GUI connection changed or failed during external creation");
                    }
                    if external.is_none() {
                        let rx: &std::sync::mpsc::Receiver<Result<ExternalWorkspace>> = external_rx.as_ref().context("missing external receiver")?;
                        match rx.try_recv() {
                            Ok(result) => external = Some(result?),
                            Err(std::sync::mpsc::TryRecvError::Empty) => {},
                            Err(error) => return Err(error).context("external worker"),
                        }
                    }
                    if since.elapsed() > Duration::from_secs(12) {
                        bail!("external workspace worker/GUI timed out");
                    }
                    let Some(created) = &external else { return Ok(false) };
                    let elapsed = created.sent.elapsed();
                    if elapsed > EXTERNAL_TIMEOUT {
                        bail!("external workspace not consumed by GUI within {EXTERNAL_TIMEOUT:?}: elapsed={elapsed:?} id={} snapshot={:?}", created.id, view.live.snapshot);
                    }
                    let Some(snapshot) = &view.live.snapshot else { return Ok(false) };
                    let before: &Arc<ClientShellSnapshot> = before;
                    if snapshot.boot_id != before.boot_id || snapshot.focused_workspace_id != before.focused_workspace_id
                        || snapshot.focused_tab_id != before.focused_tab_id || snapshot.focused_pane_id != before.focused_pane_id {
                        bail!("external unfocused creation changed GUI boot/focus");
                    }
                    if !snapshot.workspaces.iter().any(|w| w.workspace_id == created.id && w.label == "external-gui-smoke") { return Ok(false); }
                    if snapshot.revision <= before.revision || snapshot.workspaces.len() != before.workspaces.len() + 1
                        || snapshot.tabs.len() != before.tabs.len() + 1 {
                        bail!("incorrect external workspace snapshot: {snapshot:?}");
                    }
                    eprintln!("GUI external workspace push verified: id={} revision={} -> {} command_to_observed_ms={} response_to_observed_ms={} bound_ms={} observation_poll_ms=100 unchanged_connection=true unchanged_focus=true no_refresh=true",
                        created.id, before.revision, snapshot.revision, elapsed.as_millis(), created.responded.elapsed().as_millis(), EXTERNAL_TIMEOUT.as_millis());
                    return Ok(true);
                }
                if frames == 0 {
                    // Exercise the regression: no foreground app or pre-existing input focus.
                    cx.hide();
                    window.blur(cx);
                }
                // on_next_frame runs BEFORE draw, and hidden windows may not receive it.
                // Build the real native window's dispatch tree and input handler synchronously,
                // without activating the app or relying on desktop/OS keyboard focus.
                window.focus(&view.read(cx).focus.clone(), cx);
                window.refresh();
                window.draw(cx).clear(cx);
                frames += 1;
                let focused = view.read(cx).focus.is_focused(window);
                let active = window.is_window_active();
                let actions_ready = window.is_action_available(&RunCommand { command: Command::Tab }, cx);
                let probe = view.read(cx).input_probe;
                let navigations = view.read(cx).navigations.clone();
                // A command sent while a focus change is fenced is dropped,
                // as a user's would be, so each step waits for the fence.
                let fenced = view.read(cx).live.activation.fenced();
                let ready = view.read(cx).navigation_ready();
                let (live, local_error, options, last_queued_options, bounds) = {
                    let view = view.read(cx);
                    (view.live.clone(), view.notices.text().map(str::to_owned), view.options, view.last_queued_options, view.bounds)
                };
                let handoff = handoff_diagnostic(&live, &workspace, &first_tab);
                let diagnostic = || format!(
                    "step={step} ({}) elapsed={:?} {} frames={frames} focus={focused} actions_ready={actions_ready} fenced={fenced} ready={ready} active={} probe={probe:?} status={:.160} local_error={:.240} live.error={:.240} connected={} snapshot={:?} surface={:?} size={:?} last_queued_options={last_queued_options:?} navigations={navigations:?}",
                    STEPS[step], since.elapsed(), handoff, active, live.status,
                    local_error.as_deref().unwrap_or("none"), live.notices.text().unwrap_or("none"), live.status.is_connected(),
                    live.snapshot.as_ref().map(|s| (s.revision, s.workspaces.len(), s.tabs.len())),
                    live.surface.as_ref().map(|s| (s.projection_revision, s.panes.len(), s.frame.width, s.frame.height)), options.surface_size
                );
                if local_error.is_some() || live.notices.is_some() {
                    bail!(diagnostic());
                }
                if since.elapsed() > Duration::from_secs(20) {
                    // What the pane showed, so a CI timeout says what the
                    // terminal did with the input.
                    let screen = live.surface.as_ref().map(|s| screen_text(&s.frame)).unwrap_or_default();
                    bail!(timeout_diagnostic(&diagnostic(), &screen));
                }
                if !focused || !actions_ready || fenced || !ready { return Ok(false); }
                let (Some(snapshot), Some(surface)) = (&live.snapshot, &live.surface) else { return Ok(false) };
                if !live.status.is_connected() || snapshot.boot_id != surface.boot_id || snapshot.revision != surface.projection_revision {
                    return Ok(false);
                }
                surface.frame.validate().with_context(|| format!("invalid frame; {}", diagnostic()))?;
                if let Some(error) = &snapshot.config_diagnostic {
                    bail!("config diagnostic: {error:?}; {}", diagnostic());
                }
                let focused_tab = snapshot.focused_tab_id.as_deref().unwrap_or_default();
                let focused_workspace = snapshot.focused_workspace_id.as_deref().unwrap_or_default();
                // A command is typed as this platform's own default shortcut
                // (Cmd on macOS, Ctrl elsewhere) and must reach its action;
                // any other key must reach the terminal.
                let key = |input: Key, window: &mut Window, cx: &mut App| -> Result<()> {
                    let name = match input {
                        Key::Command(command) => shortcut(command)?,
                        Key::Terminal(name) => name,
                    };
                    let before = view.read(cx).input_probe;
                    let key = spaceport_keymap::parse_keystroke(name).with_context(|| format!("parsing keystroke {name}"))?;
                    if !window.dispatch_keystroke(key, cx) { bail!("unhandled keystroke {name}"); }
                    let after = view.read(cx).input_probe;
                    let delivered = if matches!(input, Key::Command(_)) {
                        after.actions == before.actions + 1
                    } else {
                        after.keys == before.keys + 1
                    };
                    if !delivered { bail!("keystroke {name} missed intended handler: before={before:?} after={after:?}; {}", diagnostic()); }
                    Ok(())
                };
                match step {
                    0 if !focused_tab.is_empty() && surface.panes.len() == 1 && bounds.size.width > px(0.) => {
                        boot = snapshot.boot_id.clone();
                        workspace = focused_workspace.into();
                        first_tab = focused_tab.into();
                        key(Key::Command(Command::Tab), window, cx)?;
                    }
                    1 if snapshot.tabs.len() == 2 && focused_tab != first_tab && surface.panes.len() == 1 => {
                        second_tab = focused_tab.into();
                        split_pane = snapshot.focused_pane_id.clone().unwrap_or_default();
                        key(Key::Command(Command::SplitRight), window, cx)?;
                    }
                    2 if surface.panes.len() == 2 => {
                        let old = surface.panes.iter().find(|p| p.pane_id == split_pane).context("original split pane missing")?;
                        let new = surface.panes.iter().find(|p| Some(&p.pane_id) == snapshot.focused_pane_id.as_ref()).context("focused split missing")?;
                        if new.rect.x <= old.rect.x || new.rect.y != old.rect.y { bail!("right split geometry: {}", diagnostic()); }
                        split_pane = new.pane_id.clone();
                        key(Key::Command(Command::SplitDown), window, cx)?;
                    }
                    3 if surface.panes.len() == 3 => {
                        let old = surface.panes.iter().find(|p| p.pane_id == split_pane).context("original split pane missing")?;
                        let new = surface.panes.iter().find(|p| Some(&p.pane_id) == snapshot.focused_pane_id.as_ref()).context("focused split missing")?;
                        if new.rect.y <= old.rect.y || new.rect.x != old.rect.x { bail!("down split geometry: {}", diagnostic()); }
                        window.dispatch_action(Box::new(RunCommand { command: Command::PreviousTab }), cx);
                    }
                    4 if focused_tab == first_tab && surface.panes.len() == 1 => {
                        window.dispatch_action(Box::new(RunCommand { command: Command::NextTab }), cx);
                    }
                    5 if focused_tab == second_tab && surface.panes.len() == 3 => {
                        key(Key::Command(Command::Workspace), window, cx)?;
                    }
                    6 if snapshot.workspaces.len() == 2 && focused_workspace != workspace && surface.panes.len() == 1 => {
                        let before = view.read(cx).presentation.probe;
                        view.update(cx, |view, cx| { view.navigate(NavigationTarget::Workspace(&workspace), cx); window.focus(&view.focus, cx); });
                        // Draw the frame that follows the focus change immediately: the client
                        // has just dropped its surface and the next projection is a round trip
                        // away, which is precisely when the terminal area used to blank.
                        window.refresh();
                        window.draw(cx).clear(cx);
                        let after = view.read(cx).presentation.probe;
                        if after.blank > before.blank {
                            bail!("space switch blanked the terminal area: {} empty frame(s); {}", after.blank - before.blank, diagnostic());
                        }
                        switched = Some((Instant::now(), after));
                    }
                    7 if focused_workspace == workspace && focused_tab == second_tab && surface.panes.len() == 3 => {
                        let (started, before) = switched.take().context("missing space switch probe")?;
                        let probe = view.read(cx).presentation.probe;
                        if probe.blank > before.blank {
                            bail!("space switch blanked the terminal area: {} empty frame(s); {}", probe.blank - before.blank, diagnostic());
                        }
                        eprintln!("GUI space switch verified: blank_frames=0 retained_paints={} gap_observed_ms={} observation_poll_ms=100",
                            probe.retained - before.retained, started.elapsed().as_millis());
                        // Use the full-width tab so the exact output row cannot wrap in a split.
                        window.dispatch_action(Box::new(RunCommand { command: Command::PreviousTab }), cx);
                    }
                    // Typed only at a prompt: a shell still starting (PowerShell
                    // under emulation on Windows on ARM) can drop typed-ahead input.
                    8 if focused_tab == first_tab && surface.panes.len() == 1 && shell_ready(&surface.frame) => {
                        let command = format!("echo HERDR_GUI_{}\"_OK\"", std::process::id());
                        type_text(&command, &view, window, cx)?;
                        key(Key::Terminal("enter"), window, cx)?;
                    }
                    9 if has_output(&surface.frame, &marker) => {
                        eprintln!("GUI shell output verified (not command echo): {marker}");
                        old_size = options.surface_size;
                        window.resize(fixture_size(1000., 650.));
                    }
                    10 if options.surface_size != old_size && last_queued_options == Some(options)
                        && surface.frame.width == options.surface_size.cols && surface.frame.height == options.surface_size.rows => {
                        eprintln!("GUI native resize verified: {:?} -> {:?}", old_size, options.surface_size);
                        view.update(cx, |view, cx| { view.reconnect(cx); window.focus(&view.focus, cx); cx.notify(); });
                    }
                    11 if snapshot.boot_id == boot && snapshot.workspaces.len() == 2 && snapshot.tabs.len() == 3
                        && focused_workspace == workspace && focused_tab == first_tab && has_output(&surface.frame, &marker) => {
                        type_text(&format!("echo HERDR_GUI_{}\"_OK_RECONNECTED\"", std::process::id()), &view, window, cx)?;
                        key(Key::Terminal("enter"), window, cx)?;
                    }
                    12 if has_output(&surface.frame, &reconnected_marker) => {
                        eprintln!("GUI input pipeline verified: frames={frames} focus={focused} active={active} probe={probe:?}");
                        eprintln!("GUI fresh input after reconnect verified: {reconnected_marker}");
                        let target = view.read(cx).endpoints[view.read(cx).selected_endpoint].connection.target.clone();
                        if !matches!(&target, ConnectTarget::Socket(_)) {
                            bail!("external smoke requires an explicit isolated socket");
                        }
                        baseline = Some((snapshot.clone(), view.read(cx).endpoints[view.read(cx).selected_endpoint].connection.inbox.clone()));
                        let boot = boot.clone();
                        let (tx, rx) = std::sync::mpsc::channel();
                        std::thread::Builder::new().name("external-workspace-smoke".into()).spawn(move || {
                            let _ = tx.send(create_external_workspace(target, options, boot));
                        }).context("spawning external workspace worker")?;
                        external_rx = Some(rx);
                    }
                    _ => return Ok(false),
                }
                eprintln!("GUI step {step} ({}) verified; waiting for {}", STEPS[step], STEPS[step + 1]);
                step += 1;
                since = Instant::now();
                Ok(false)
            });
            match result {
                Ok(Ok(true)) => {
                    completed = true;
                    break;
                }
                Ok(Ok(false)) => {},
                error => {
                    set_exit_code(1);
                    eprintln!("GUI integration FAIL: {error:?}");
                    cx.update(|cx| cx.quit());
                    break;
                }
            }
        }
        if completed {
            #[cfg(target_os = "macos")]
            if let Err(error) = clipboard::verify(handle, cx).await {
                set_exit_code(1);
                eprintln!("GUI clipboard FAIL: {error:#}");
                cx.update(|cx| cx.quit());
                return;
            }
            #[cfg(target_os = "macos")]
            if let Err(error) = selection::verify(handle, cx).await {
                set_exit_code(1);
                eprintln!("GUI selection FAIL: {error:#}");
                cx.update(|cx| cx.quit());
                return;
            }
            match second_window::verify(handle, cx).await {
                Ok(()) => {
                    #[cfg(target_os = "macos")]
                    if let Err(error) = clipboard::verify_remote(cx).await {
                        set_exit_code(1);
                        eprintln!("GUI remote clipboard FAIL: {error:#}");
                        cx.update(|cx| cx.quit());
                        return;
                    }
                    eprintln!("GUI integration PASS: same boot={boot}, 3 workspaces / 4 tabs, persisted shell output after reconnect, external workspace pushed to idle GUI, second window on its own space");
                    set_exit_code(0);
                }
                Err(error) => {
                    set_exit_code(1);
                    eprintln!("GUI second window FAIL: {error:#}");
                }
            }
            cx.update(|cx| cx.quit());
        }
    }).detach();
}

#[cfg(test)]
#[path = "smoke_live/tests.rs"]
mod tests;
