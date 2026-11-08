//! Tabs opened in the caller's workspace: Devices and px0.
//! Each feature opens its tab in the focused
//! workspace, so the window first switches to the caller's, and the tab
//! opens once a snapshot says the switch landed.
use super::{busy, closed};
use crate::{
    SheprdWindow,
    browser::{Location, Store},
    control::{
        protocol::{
            Caller, ErrorCode, Response,
            tabs::{ARRIVE_WAIT_SECONDS, Reply},
        },
        server::{Watches, socket, target, window_for},
    },
};
use gpui::{App, Context, Window, WindowHandle};
use std::time::Duration;

/// The window, endpoint, and workspace a request's tab goes to.
struct Destination {
    handle: WindowHandle<SheprdWindow>,
    endpoint: usize,
    workspace: String,
}

fn destination(caller: &Caller, cx: &mut App) -> Result<Destination, Response> {
    let handle = window_for(caller, cx)?;
    let target = target(caller);
    let found = handle
        .read_with(cx, |view, _| {
            target
                .strictness()
                .iter()
                .find_map(|strict| view.control_target(&target, *strict))
        })
        .map_err(|_| closed())?;
    let (endpoint, workspace) = found.ok_or_else(|| {
        Response::error(
            ErrorCode::WorkspaceNotFound,
            match &caller.workspace_id {
                Some(id) => format!("No Spaceport window shows workspace {id}"),
                None => "The window shows no workspace yet".to_owned(),
            },
        )
    })?;
    Ok(Destination {
        handle,
        endpoint,
        workspace,
    })
}

/// Switches the window to the caller's workspace, then, once the switch
/// landed and `ready` holds, runs `act` there and answers with what it says.
fn open_there(
    caller: &Caller,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
    ready: impl Fn(&SheprdWindow) -> bool + 'static,
    act: impl FnOnce(&mut SheprdWindow, &str, &mut Window, &mut Context<SheprdWindow>) -> Response
    + 'static,
) {
    let Destination {
        handle,
        endpoint,
        workspace,
    } = match destination(caller, cx) {
        Ok(destination) => destination,
        Err(response) => return incoming.respond(response),
    };
    let started = handle
        .update(cx, |view, _, cx| {
            if view.menu.page.is_some() {
                return Err(busy());
            }
            view.go_to_workspace(endpoint, &workspace, cx);
            Ok(())
        })
        .unwrap_or_else(|_| Err(closed()));
    if let Err(response) = started {
        return incoming.respond(response);
    }
    let expired = Response::error(
        ErrorCode::Timeout,
        format!("Workspace {workspace} was not ready in the window in time"),
    );
    let mut act = Some(act);
    watches.add(
        incoming,
        Duration::from_secs(ARRIVE_WAIT_SECONDS),
        move |cx| {
            let arrived = handle.read_with(cx, |view, _| {
                view.shows_workspace(endpoint, &workspace) && ready(view)
            });
            match arrived {
                Err(_) => return Some(closed()),
                Ok(false) => return None,
                Ok(true) => {}
            }
            let act = act.take()?;
            Some(
                handle
                    .update(cx, |view, window, cx| {
                        if view.menu.page.is_some() {
                            return busy();
                        }
                        act(view, &workspace, window, cx)
                    })
                    .unwrap_or_else(|_| closed()),
            )
        },
        expired,
        cx,
    );
}

/// The tab of the window's focused workspace that `shows` picks, as the
/// answer to opening it; the feature already told the user why when none.
fn opened(
    view: &SheprdWindow,
    workspace: &str,
    what: &str,
    cx: &App,
    shows: impl Fn(&Location) -> bool,
) -> Response {
    let scope = crate::browser::scope(&view.endpoints[view.selected_endpoint]);
    let id = cx.try_global::<Store>().and_then(|store| {
        store
            .in_workspace(&scope, workspace)
            .filter(|tab| !tab.place.is_editor())
            .filter(|tab| tab.location.as_ref().is_some_and(&shows))
            .last()
            .map(|tab| tab.id)
    });
    match id {
        Some(id) => Response::Tab {
            reply: Reply::Opened {
                id: id.into(),
                workspace: workspace.to_owned(),
            },
        },
        None => Response::error(
            ErrorCode::Failed,
            format!("Could not open {what}; the window says why"),
        ),
    }
}

pub(super) fn devices(
    caller: &Caller,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
) {
    open_there(
        caller,
        incoming,
        watches,
        cx,
        |_| true,
        |view, workspace, window, cx| {
            view.open_devices_overview(window, cx);
            opened(view, workspace, "the Devices overview", cx, |location| {
                matches!(location, Location::Devices)
            })
        },
    );
}

pub(super) fn px0(
    caller: &Caller,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
) {
    open_there(
        caller,
        incoming,
        watches,
        cx,
        |_| true,
        |view, workspace, window, cx| {
            if !crate::browser::EMBEDDED || view.px0_root().is_none() {
                return Response::error(
                    ErrorCode::Unsupported,
                    "px0 needs a local workspace and embedded pages (macOS or Windows)",
                );
            }
            match view.open_px0(None, window, cx) {
                Some(id) => Response::Tab {
                    reply: Reply::Opened {
                        id: id.into(),
                        workspace: workspace.to_owned(),
                    },
                },
                None => Response::error(ErrorCode::Failed, "Could not open the px0 tab"),
            }
        },
    );
}
