// ABOUTME: Lists and targets native app tabs through the local control socket.
// ABOUTME: Preserves saved Core references while refusing their actions when Core is off.
//! `tab` and `devices` requests.
//! The Store keeps app tabs; caller targets still select their workspace.
mod arrive;

use super::{Watches, not_found, session_window, socket, valid_caller};
use crate::{
    SheprdWindow,
    browser::{Location, Store, Tab, TabId},
    control::protocol::{
        Call, Caller, ErrorCode, Response,
        tabs::{Reply, TabInfo, TabKind, TabOp},
    },
};
use gpui::{App, Context, Window, WindowHandle};

pub(super) fn serve(
    call: Call<TabOp>,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
) {
    let caller = call.caller;
    if !valid_caller(&caller) {
        incoming.respond(Response::error(
            ErrorCode::InvalidRequest,
            "Invalid workspace, pane, or daemon",
        ));
        return;
    }
    match call.op {
        TabOp::List {} => incoming.respond(list_for(&caller, cx)),
        TabOp::Focus { id } => incoming.respond(focus(&caller, id, cx)),
        TabOp::Close { id } => incoming.respond(close(&caller, id, cx)),
        TabOp::Devices => arrive::devices(&caller, incoming, watches, cx),
    }
}

/// What a tab is, as `tab list` names it.
fn kind(tab: &Tab) -> Option<TabKind> {
    match &tab.location {
        None if tab.core_group().is_some() => Some(TabKind::CoreTerminal),
        // A blank tab outside a Core group has nothing to name.
        None => None,
        Some(Location::Devices) => Some(TabKind::Devices),
        Some(Location::CoreTerminal { .. }) => Some(TabKind::CoreTerminal),
    }
}

/// The runtime terminal IDs a Core tab shows.
fn location(tab: &Tab) -> Option<String> {
    match tab.location.as_ref()? {
        // Every pane's terminal, for `herdr-core attach` and the like.
        Location::CoreTerminal { panes, .. } => Some(
            panes
                .terminals()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
        Location::Devices => None,
    }
}

fn info(tab: &Tab) -> Option<TabInfo> {
    Some(TabInfo {
        id: tab.id.into(),
        kind: kind(tab)?,
        title: tab.title.clone(),
        location: location(tab),
        workspace: tab.workspace_id.clone(),
        host: tab.scope.as_str().to_owned(),
        opened_by: tab.origin.clone(),
    })
}

fn list_for(caller: &Caller, cx: &mut App) -> Response {
    let handle = match session_window(caller, cx) {
        Ok(handle) => handle,
        Err(response) => return response,
    };
    list_in(caller, handle, cx)
}

#[cfg(test)]
fn list(cx: &mut App) -> Response {
    list_for(&Caller::default(), cx)
}

fn list_in(caller: &Caller, handle: WindowHandle<SheprdWindow>, cx: &App) -> Response {
    let tabs = cx
        .try_global::<Store>()
        .map(|store| {
            store
                .tabs()
                .filter(|tab| !tab.requires_core() || crate::core_feature::enabled(cx))
                .filter(|tab| showing_in(caller, handle, tab, cx).is_some())
                .filter_map(info)
                .collect()
        })
        .unwrap_or_default();
    Response::Tab {
        reply: Reply::Tabs { tabs },
    }
}

fn find(id: u64, cx: &App) -> Result<Tab, Response> {
    let tab = cx
        .try_global::<Store>()
        .and_then(|store| store.get(TabId::from(id)))
        .filter(|tab| kind(tab).is_some())
        .cloned()
        .ok_or_else(|| not_found(format!("Spaceport has no tab {id}; `tab list` lists them")))?;
    if tab.requires_core() && !crate::core_feature::enabled(cx) {
        return Err(Response::error(
            ErrorCode::Unsupported,
            crate::core_feature::disabled_message(),
        ));
    }
    Ok(tab)
}

fn closed() -> Response {
    Response::error(ErrorCode::NoWindow, "The window closed")
}

fn busy() -> Response {
    Response::error(
        ErrorCode::WindowBusy,
        "A dialog is open in the window; close it first",
    )
}

struct Showing {
    window: WindowHandle<SheprdWindow>,
    endpoint: Option<usize>,
}

/// Checks membership in one resolved window and the caller's exact session.
fn showing_in(
    caller: &Caller,
    handle: WindowHandle<SheprdWindow>,
    tab: &Tab,
    cx: &App,
) -> Option<Showing> {
    if caller
        .workspace_id
        .as_ref()
        .is_some_and(|workspace| workspace != &tab.workspace_id)
    {
        return None;
    }
    handle
        .read_with(cx, |view, cx| {
            let key = (tab.scope.clone(), tab.workspace_id.clone());
            if view
                .browser
                .hidden
                .hides(&key, &crate::browser::Pick::Page(tab.id))
            {
                return None;
            }
            if let Some(key) = tab.core_group() {
                if caller.workspace_id.is_some() || caller.daemon_socket.is_some() {
                    return None;
                }
                return view.core_group_available(&key, cx).then_some(Showing {
                    window: handle,
                    endpoint: None,
                });
            }
            let index = view.endpoint_showing(&tab.scope, &tab.workspace_id)?;
            let caller_endpoint = if caller.workspace_id.is_some() || caller.daemon_socket.is_some()
            {
                let target = super::target(caller);
                target
                    .strictness()
                    .iter()
                    .find_map(|strict| view.control_target(&target, *strict))?
                    .0
            } else {
                view.selected_endpoint
            };
            if index != caller_endpoint {
                return None;
            }
            Some(Showing {
                window: handle,
                endpoint: Some(index),
            })
        })
        .ok()
        .flatten()
}

fn showing(caller: &Caller, tab: &Tab, cx: &mut App) -> Result<Showing, Response> {
    let handle = session_window(caller, cx)?;
    showing_in(caller, handle, tab, cx)
        .ok_or_else(|| not_found(format!("The selected session does not show tab {}", tab.id)))
}

/// Validates an existing tab before starting a closure wait.
pub(super) fn wait_target(
    caller: &Caller,
    id: u64,
    cx: &mut App,
) -> Result<WindowHandle<SheprdWindow>, Response> {
    let tab = find(id, cx)?;
    Ok(showing(caller, &tab, cx)?.window)
}

/// Switches a window to `tab`'s workspace and shows the tab there. Shown
/// against its workspace, the tab is in front once the switch lands.
fn reveal(caller: &Caller, tab: &Tab, cx: &mut App) -> Result<(), Response> {
    let showing = showing(caller, tab, cx)?;
    showing
        .window
        .update(cx, |view, window, cx| {
            if view.menu.page.is_some() {
                return Err(busy());
            }
            if let Some(key) = tab.core_group() {
                if !view.show_core_group(&key, cx) {
                    return Err(not_found(format!(
                        "Core group {} is unavailable",
                        key.group_id()
                    )));
                }
            } else if let Some(index) = showing.endpoint {
                view.go_to_workspace(index, &tab.workspace_id, cx);
            }
            view.show_browser_tab(tab.id, window, cx);
            Ok(())
        })
        .unwrap_or_else(|_| Err(closed()))
}

fn focus(caller: &Caller, id: u64, cx: &mut App) -> Response {
    let tab = match find(id, cx) {
        Ok(tab) => tab,
        Err(response) => return response,
    };
    match reveal(caller, &tab, cx) {
        Ok(()) => Response::done(format!(
            "Showing tab {id} ({}) in workspace {}",
            tab.title, tab.workspace_id
        )),
        Err(response) => response,
    }
}

/// Closes only a tab available in the resolved caller window and session.
fn close(caller: &Caller, id: u64, cx: &mut App) -> Response {
    let tab = match find(id, cx) {
        Ok(tab) => tab,
        Err(response) => return response,
    };
    let showing = match showing(caller, &tab, cx) {
        Ok(showing) => showing,
        Err(response) => return response,
    };
    let closed = showing.window.update(cx, |view, window, cx| {
        if view.menu.page.is_some() {
            return Err(busy());
        }
        Ok(close_in_window(view, &tab, window, cx))
    });
    match closed {
        Ok(Ok(true)) => {}
        Ok(Err(response)) => return response,
        _ => return not_found(format!("No Spaceport window shows tab {id}")),
    }
    Response::done(format!("Closed tab {id} ({})", tab.title))
}

fn close_in_window(
    view: &mut SheprdWindow,
    tab: &Tab,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> bool {
    if let Some(key) = tab.core_group()
        && !view.core_group_available(&key, cx)
    {
        return false;
    }
    view.close_browser_tab(tab.id, window, cx);
    true
}

#[cfg(test)]
mod tests;
