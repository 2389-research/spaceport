//! `status`, `app`, `window`, `ui`, and `run`.
use super::{in_free_window, in_window, main_windows, window_for};
use crate::{
    SheprdWindow,
    app::WindowLaunch,
    constants::APP_VERSION,
    control::protocol::{
        Call, Caller, ErrorCode, PROTOCOL_VERSION, Response,
        app::{
            AppOp, CommandOp, CoreCatalog, CoreCatalogState, CoreWorkspaceInfo, FontSize, HostInfo,
            Reply, Status, UiOp, WindowInfo, WindowOp,
        },
    },
    controls::{COMMANDS, Command},
    core_terminals,
};
use gpui::{App, Window, px, size};
use sheprd_client::ConnectTarget;
use std::time::Duration;

/// How long `app quit` waits, so its answer reaches the caller first.
const QUIT_DELAY: Duration = Duration::from_millis(200);

impl SheprdWindow {
    /// The hosts this window shows, with their connection status.
    pub(crate) fn control_hosts(&self, cx: &App) -> Vec<HostInfo> {
        let mutes = crate::notification_mutes::Mutes::of(cx);
        self.endpoints
            .iter()
            .enumerate()
            .map(|(index, endpoint)| {
                let selected = index == self.selected_endpoint;
                // The selected host's live state is the window's own.
                let status = match endpoint.status() {
                    "online" | "connecting" if selected => {
                        if self.live.status.is_connected() {
                            "online"
                        } else {
                            "connecting"
                        }
                    }
                    status => status,
                };
                HostInfo {
                    id: endpoint.id.clone(),
                    label: endpoint.label.clone(),
                    status: status.to_owned(),
                    selected,
                    muted: mutes.is_some_and(|mutes| mutes.host(&endpoint.id)),
                    tags: self.device_tags.of(&endpoint.id).to_vec(),
                }
            })
            .collect()
    }

    fn control_info(&self, window: &Window, cx: &App) -> WindowInfo {
        let bounds = window.bounds();
        let core_group = self.selected_core_group();
        let workspace = if core_group.is_some() {
            None
        } else {
            self.live
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.focused_workspace_id.clone())
        };
        WindowInfo {
            id: window.window_handle().window_id().as_u64(),
            title: self.title.clone(),
            active: window.is_window_active(),
            fullscreen: window.is_fullscreen(),
            width: f32::from(bounds.size.width),
            height: f32::from(bounds.size.height),
            workspace,
            core_workspace: core_group.map(|key| CoreWorkspaceInfo {
                hub_core_id: key.hub_core_id().to_owned(),
                machine_id: key.machine_id().map(str::to_owned),
                document_name: key.document_name().to_owned(),
                group_id: key.group_id().to_string(),
                current: self.core_group_available(key, cx),
            }),
            hosts: self.control_hosts(cx),
        }
    }
}

fn windows(cx: &mut App) -> Vec<WindowInfo> {
    main_windows(cx)
        .into_iter()
        .filter_map(|handle| {
            handle
                .update(cx, |view, window, cx| view.control_info(window, cx))
                .ok()
        })
        .collect()
}

pub(super) fn status(_: &Caller, cx: &mut App) -> Response {
    let epoch = core_terminals::machine_catalog_epoch(cx);
    let machine_count = core_terminals::machine_catalog(cx).map(|machines| machines.len());
    let error = core_terminals::machine_catalog_error_code(cx);
    let state = if epoch.is_none() {
        CoreCatalogState::NotStarted
    } else if core_terminals::machine_catalog_stale(cx) {
        CoreCatalogState::Disconnected
    } else if !core_terminals::machine_catalog_fresh(cx) {
        CoreCatalogState::Synchronizing
    } else if machine_count.is_some() {
        CoreCatalogState::Current
    } else if error.is_some() {
        CoreCatalogState::Unavailable
    } else {
        CoreCatalogState::Synchronizing
    };
    let machine_count = if state == CoreCatalogState::Current {
        machine_count
    } else {
        None
    };
    Response::Status(Status {
        version: APP_VERSION.to_owned(),
        protocol: PROTOCOL_VERSION,
        pid: std::process::id(),
        windows: windows(cx),
        core_catalog: CoreCatalog {
            epoch,
            state,
            machine_count,
            error: error.map(str::to_owned),
        },
    })
}

/// Brings `handle`'s window forward. Deferred: activating a window updates
/// it, which cannot happen inside another update.
fn activate(handle: gpui::WindowHandle<SheprdWindow>, cx: &mut App) {
    cx.activate(true);
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, _| window.activate_window());
    });
}

pub(super) fn app(call: Call<AppOp>, cx: &mut App) -> Response {
    match call.op {
        AppOp::Focus => match window_for(&call.caller, cx) {
            Ok(handle) => {
                activate(handle, cx);
                Response::done("Brought Spaceport forward")
            }
            Err(response) => response,
        },
        AppOp::Hide => {
            cx.hide();
            Response::done("Hid Spaceport")
        }
        AppOp::Quit => {
            let timer = cx.background_executor().clone();
            cx.spawn(async move |cx| {
                timer.timer(QUIT_DELAY).await;
                cx.update(|cx| cx.quit());
            })
            .detach();
            Response::done("Quitting Spaceport; Herdr and its terminals keep running")
        }
    }
}

/// The selected Core group or legacy launch endpoint of a window.
fn launch_for_window(view: &SheprdWindow, cx: &App) -> Result<WindowLaunch, Response> {
    if let Some(key) = view.selected_core_group() {
        return view
            .core_group_available(key, cx)
            .then(|| WindowLaunch::Core(key.clone()))
            .ok_or_else(|| {
                Response::error(
                    ErrorCode::NotFound,
                    "Selected Core group is no longer available",
                )
            });
    }
    view.endpoints
        .first()
        .map(|endpoint| WindowLaunch::Legacy(endpoint.connection.target.clone()))
        .ok_or_else(|| Response::error(ErrorCode::NotFound, "Window has no launch target"))
}

/// `--window` names a window's exact selected target. Otherwise a daemon
/// socket names the caller's legacy host; outside a pane, use the frontmost
/// window's selected target or the local default when no window exists.
fn new_window_target(caller: &Caller, cx: &mut App) -> Result<WindowLaunch, Response> {
    if caller.window.is_some() {
        let handle = window_for(caller, cx)?;
        return handle
            .read_with(cx, launch_for_window)
            .map_err(|_| Response::error(ErrorCode::NoWindow, "The window closed"))?;
    }
    if let Some(daemon) = caller.daemon_socket.as_deref() {
        let daemon = std::path::Path::new(daemon);
        let known = main_windows(cx).into_iter().find_map(|handle| {
            handle
                .read_with(cx, |view, _| {
                    view.endpoints
                        .iter()
                        .find(|endpoint| {
                            endpoint.connection.target.socket_path().ok().as_deref() == Some(daemon)
                        })
                        .map(|endpoint| endpoint.connection.target.clone())
                })
                .ok()
                .flatten()
        });
        return Ok(WindowLaunch::Legacy(
            known.unwrap_or_else(|| ConnectTarget::Socket(daemon.to_owned())),
        ));
    }
    let Some(handle) = main_windows(cx).first().copied() else {
        return Ok(WindowLaunch::Legacy(ConnectTarget::Local));
    };
    handle
        .read_with(cx, launch_for_window)
        .map_err(|_| Response::error(ErrorCode::NoWindow, "The window closed"))?
}

pub(super) fn window(call: Call<WindowOp>, cx: &mut App) -> Response {
    let caller = &call.caller;
    match call.op {
        WindowOp::List => Response::App {
            reply: Reply::Windows {
                windows: windows(cx),
            },
        },
        WindowOp::New {} => {
            let target = match new_window_target(caller, cx) {
                Ok(target) => target,
                Err(response) => return response,
            };
            match target {
                WindowLaunch::Legacy(target) => crate::open_additional_window(target, cx),
                WindowLaunch::Core(key) => crate::app::open_additional_core_window(key, cx),
            }
            // Opened after this answer; a failure is only logged.
            Response::done("Asked for a new window; `window list` shows it once open")
        }
        WindowOp::Focus => match window_for(caller, cx) {
            Ok(handle) => {
                activate(handle, cx);
                Response::done(format!("Focused window {}", handle.window_id().as_u64()))
            }
            Err(response) => response,
        },
        WindowOp::Fullscreen { switch } => in_window(caller, cx, |_, window, _| {
            let now = window.is_fullscreen();
            if switch.apply(now) != now {
                window.toggle_fullscreen();
            }
            Response::done(if switch.apply(now) {
                "Full screen"
            } else {
                "Not full screen"
            })
        }),
        WindowOp::Size { size: None } => in_window(caller, cx, |_, window, _| {
            let bounds = window.bounds();
            Response::App {
                reply: Reply::Size {
                    width: f32::from(bounds.size.width),
                    height: f32::from(bounds.size.height),
                },
            }
        }),
        WindowOp::Size { size: Some(asked) } => in_window(caller, cx, |_, window, _| {
            window.resize(size(px(asked.width as f32), px(asked.height as f32)));
            Response::done(format!("Resized to {}x{}", asked.width, asked.height))
        }),
        WindowOp::Detach => in_free_window(caller, cx, |view, window, cx| {
            view.command(Command::Detach, window, cx);
            Response::done("Detached from the selected host; it keeps running")
        }),
        WindowOp::Reconnect => in_free_window(caller, cx, |view, window, cx| {
            view.command(Command::Reconnect, window, cx);
            Response::done("Reconnecting to the selected host")
        }),
    }
}

pub(super) fn ui(call: Call<UiOp>, cx: &mut App) -> Response {
    in_window(&call.caller, cx, |view, window, cx| match call.op {
        UiOp::Sidebar { switch } => {
            let visible = switch.apply(view.sidebar_visible);
            if visible != view.sidebar_visible {
                view.toggle_sidebar();
                cx.notify();
            }
            Response::done(if visible {
                "Sidebar shown"
            } else {
                "Sidebar hidden"
            })
        }
        UiOp::Panel { panel } => {
            if !view.activity_bar_shown() {
                return Response::error(
                    ErrorCode::Unsupported,
                    "The activity bar is off: [sidebar] activity_bar = false",
                );
            }
            if !view.activity_panel_listed(panel) {
                let why = format!(
                    "{} is turned off: [sidebar] {} = false",
                    panel.name(),
                    panel.name()
                );
                return Response::error(ErrorCode::Unsupported, why);
            }
            view.show_activity_panel(panel, window, cx);
            Response::done(format!("Showing {}", panel.name()))
        }
        UiOp::StatusBar { switch } => {
            let visible = switch.apply(view.status_bar_visible);
            if visible != view.status_bar_visible {
                view.status_bar_visible = visible;
                cx.notify();
            }
            Response::done(if visible {
                "Status bar shown"
            } else {
                "Status bar hidden"
            })
        }
        UiOp::FontSize { size } => {
            let step = crate::config::FONT_SIZE_STEP;
            let points = match size {
                FontSize::Larger => view.config.terminal.size + step,
                FontSize::Smaller => view.config.terminal.size - step,
                FontSize::Reset => view.configured_terminal_size,
                FontSize::Points { points } => f32::from(points),
            };
            view.set_terminal_font_size(points, cx);
            Response::App {
                reply: Reply::FontSize {
                    points: view.config.terminal.size,
                },
            }
        }
    })
}

pub(super) fn command(call: Call<CommandOp>, cx: &mut App) -> Response {
    let CommandOp::Run { name } = call.op;
    let Some(info) = COMMANDS.iter().find(|info| info.name == name) else {
        return super::not_found(format!("No command is named {name}"));
    };
    match info.command {
        // Quitting answers first, as `app quit` does.
        Command::Quit => app(
            Call {
                caller: call.caller,
                op: AppOp::Quit,
            },
            cx,
        ),
        command => in_free_window(&call.caller, cx, |view, window, cx| {
            view.command(command, window, cx);
            Response::done(format!("Ran {}", crate::i18n::command_english(info)))
        }),
    }
}

#[cfg(test)]
mod tests;
