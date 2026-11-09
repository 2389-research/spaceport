//! `tab` and `devices` requests.
//! The Store keeps app tabs; caller targets still select their workspace.
mod arrive;

use super::{Watches, main_windows, not_found, socket, valid_caller};
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
        TabOp::List {} => incoming.respond(list(cx)),
        TabOp::Focus { id } => incoming.respond(focus(&caller, id, cx)),
        TabOp::Close { id } => incoming.respond(close(&caller, id, cx)),
        TabOp::Devices => arrive::devices(&caller, incoming, watches, cx),
    }
}

/// What a tab is, as `tab list` names it.
fn kind(tab: &Tab) -> Option<TabKind> {
    if tab.place.is_editor() {
        return None;
    }
    match &tab.location {
        None if tab.core_group().is_some() => Some(TabKind::CoreTerminal),
        Some(Location::Devices) => Some(TabKind::Devices),
        Some(Location::CoreTerminal { .. }) => Some(TabKind::CoreTerminal),
        _ => None,
    }
}

/// The runtime terminal IDs a Core tab shows.
fn location(tab: &Tab) -> Option<String> {
    Some(match tab.location.as_ref()? {
        Location::Web { .. }
        | Location::Local { .. }
        | Location::RetiredSource { .. }
        | Location::RetiredReview { .. }
        | Location::RetiredOrchestrator { .. } => return None,
        // Every pane's terminal, for `herdr-core attach` and the like.
        Location::CoreTerminal { panes, .. } => panes
            .terminals()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        Location::Devices => return None,
    })
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

fn list(cx: &App) -> Response {
    let tabs = cx
        .try_global::<Store>()
        .map(|store| store.tabs().filter_map(info).collect())
        .unwrap_or_default();
    Response::Tab {
        reply: Reply::Tabs { tabs },
    }
}

fn find(id: u64, cx: &App) -> Result<Tab, Response> {
    cx.try_global::<Store>()
        .and_then(|store| store.get(TabId::from(id)))
        .filter(|tab| kind(tab).is_some())
        .cloned()
        .ok_or_else(|| not_found(format!("Spaceport has no tab {id}; `tab list` lists them")))
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

/// The windows a request may act in: the one `--window` names, else all,
/// frontmost first.
fn windows(caller: &Caller, cx: &App) -> Vec<WindowHandle<SheprdWindow>> {
    let mut windows = main_windows(cx);
    if let Some(id) = caller.window {
        windows.retain(|handle| handle.window_id().as_u64() == id);
    }
    windows
}

struct Showing {
    window: WindowHandle<SheprdWindow>,
    endpoint: Option<usize>,
}

/// A window listing the workspace `tab` belongs to, and the endpoint there.
fn showing(caller: &Caller, tab: &Tab, cx: &App) -> Option<Showing> {
    windows(caller, cx).into_iter().find_map(|handle| {
        if let Some(key) = tab.core_group() {
            let available = handle
                .read_with(cx, |view, cx| view.core_group_available(&key, cx))
                .ok()?;
            return available.then_some(Showing {
                window: handle,
                endpoint: None,
            });
        }
        let index = handle
            .read_with(cx, |view, _| {
                view.endpoint_showing(&tab.scope, &tab.workspace_id)
            })
            .ok()
            .flatten()?;
        Some(Showing {
            window: handle,
            endpoint: Some(index),
        })
    })
}

/// Switches a window to `tab`'s workspace and shows the tab there. Shown
/// against its workspace, the tab is in front once the switch lands.
fn reveal(caller: &Caller, tab: &Tab, cx: &mut App) -> Result<(), Response> {
    let Some(showing) = showing(caller, tab, cx) else {
        return Err(not_found(format!(
            "No Spaceport window shows workspace {}, which tab {} belongs to",
            tab.workspace_id, tab.id
        )));
    };
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

/// Closes a tab in the window showing its workspace, which moves its group
/// to another tab; with no such window, in the store, which every window
/// follows.
fn close(caller: &Caller, id: u64, cx: &mut App) -> Response {
    let tab = match find(id, cx) {
        Ok(tab) => tab,
        Err(response) => return response,
    };
    match showing(caller, &tab, cx) {
        Some(showing) => {
            let closed = showing.window.update(cx, |view, window, cx| {
                close_in_window(view, &tab, window, cx)
            });
            if !matches!(closed, Ok(true)) {
                return not_found(format!("No Spaceport window shows tab {id}"));
            }
        }
        // A named window must be the one showing the tab: a stale or wrong
        // `--window` never closes a tab elsewhere.
        None => {
            if let Some(window) = caller.window {
                return not_found(format!("Window {window} does not show tab {id}"));
            }
        }
    }
    Store::update(cx, |store| store.close(tab.id));
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
