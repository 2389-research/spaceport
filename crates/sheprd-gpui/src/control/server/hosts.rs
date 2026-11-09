//! `host`, `ports`, `usage`, and what the
//! window gathers about its hosts, read from the state it already keeps.
//! Nothing here starts a lookup, a scan, or a sample of its own; a request
//! answers from the latest one, and says why when there is none.
mod mute;
mod ports;

#[cfg(test)]
mod tests;

use super::{Watches, in_free_window, in_window, socket, target, window_for};
use crate::{
    SheprdWindow,
    browser::PageOpened,
    control::protocol::{
        Call, Caller, ErrorCode, Response,
        hosts::{HostOp, Reply, UsageInfo, UsageWindow},
    },
};
use gpui::App;
use std::time::SystemTime;

pub(super) fn serve(
    call: Call<HostOp>,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
) {
    let caller = &call.caller;
    let response = match call.op {
        HostOp::List => in_window(caller, cx, |view, _, cx| Response::Host {
            reply: Reply::Hosts {
                hosts: view.control_hosts(cx),
            },
        }),
        HostOp::Mute { host, muted } => in_window(caller, cx, |view, _, cx| {
            mute::set(view, caller, host.as_deref(), muted, cx)
        }),
        HostOp::Select { id } => in_free_window(caller, cx, |view, _, cx| {
            select(view, &id, |view, id| view.select_endpoint(id, cx))
        }),
        HostOp::Tag { id, tags } => in_window(caller, cx, |view, _, cx| tag(view, &id, tags, cx)),
        HostOp::Ports => in_window(caller, cx, |view, _, _| ports::list(view, caller)),
        HostOp::Forward { port } => in_window(caller, cx, |view, _, cx| {
            ports::forward(view, caller, port, cx)
        }),
        HostOp::Unforward { port } => in_window(caller, cx, |view, _, cx| {
            ports::unforward(view, caller, port, cx)
        }),
        HostOp::OpenPort { port } => return ports::open(call.caller, port, incoming, watches, cx),
        HostOp::Usage => in_window(caller, cx, |view, _, _| usage(view, SystemTime::now())),
    };
    incoming.respond(response);
}

/// The endpoint and workspace a request is about: the caller's own workspace
/// when this window shows it, else the window's focused workspace on its
/// selected host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Place {
    pub index: usize,
    pub endpoint: String,
    pub label: String,
    pub workspace: String,
}

impl Place {
    pub(super) fn of(view: &SheprdWindow, caller: &Caller) -> Result<Self, Response> {
        let target = target(caller);
        let (index, workspace) = target
            .strictness()
            .iter()
            .find_map(|strict| view.control_target(&target, *strict))
            .ok_or_else(|| {
                Response::error(
                    ErrorCode::WorkspaceNotFound,
                    match &caller.workspace_id {
                        Some(id) => format!("This window does not show workspace {id}"),
                        None => "The window shows no workspace".into(),
                    },
                )
            })?;
        let endpoint = &view.endpoints[index];
        Ok(Self {
            index,
            endpoint: endpoint.id.clone(),
            label: endpoint.label.clone(),
            workspace,
        })
    }
}

/// The host named by ID, else by label.
fn named<'a>(
    view: &'a SheprdWindow,
    name: &str,
) -> Result<&'a crate::endpoint::Endpoint, Response> {
    view.endpoints
        .iter()
        .find(|endpoint| endpoint.id == name)
        .or_else(|| {
            view.endpoints
                .iter()
                .find(|endpoint| endpoint.label.eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| {
            Response::error(
                ErrorCode::NotFound,
                format!("No host is named {name}; `spaceport host list` lists them"),
            )
        })
}

/// Replaces the tags of the host named by ID or label.
fn tag(view: &mut SheprdWindow, name: &str, tags: Vec<String>, cx: &mut App) -> Response {
    let endpoint = match named(view, name) {
        Ok(endpoint) => endpoint,
        Err(response) => return response,
    };
    let (id, label) = (endpoint.id.clone(), endpoint.label.clone());
    crate::device_tags::DeviceTags::set(cx, &id, tags);
    Response::done(format!("Tagged {label}"))
}

/// Shows the host named by ID or label. `choose` is the window's own
/// selection, as clicking the host in the sidebar does.
fn select(
    view: &mut SheprdWindow,
    name: &str,
    choose: impl FnOnce(&mut SheprdWindow, &str) -> bool,
) -> Response {
    let endpoint = match named(view, name) {
        Ok(endpoint) => endpoint,
        Err(response) => return response,
    };
    let (id, label) = (endpoint.id.clone(), endpoint.label.clone());
    if !endpoint.enabled {
        return Response::error(
            ErrorCode::InvalidRequest,
            format!("Host {label} is turned off; turn it on in the sidebar first"),
        );
    }
    let before = view.local_error.clone();
    if choose(view, &id) {
        return Response::done(format!("Showing {label}"));
    }
    let reason = view
        .local_error
        .clone()
        .filter(|error| Some(error) != before.as_ref())
        .unwrap_or_else(|| format!("Spaceport could not switch to {label}"));
    Response::error(ErrorCode::Failed, reason)
}

fn selected_label(view: &SheprdWindow) -> String {
    view.endpoints
        .get(view.selected_endpoint)
        .map(|endpoint| endpoint.label.clone())
        .unwrap_or_default()
}

/// Plan usage on the selected host, as the status bar shows it.
fn usage(view: &SheprdWindow, now: SystemTime) -> Response {
    let host = selected_label(view);
    let entry = view.usage.current();
    let usage: Vec<UsageInfo> = entry
        .map(|entry| {
            entry
                .readings
                .iter()
                .map(|reading| usage_info(reading, now))
                .collect()
        })
        .unwrap_or_default();
    let note = if !view.config.usage.show {
        Some("Usage is turned off ([usage] show in the settings)".to_owned())
    } else if let Some(error) = entry.and_then(|entry| entry.error.as_deref()) {
        Some(error.to_string())
    } else if usage.is_empty() {
        Some(format!(
            "Plan usage is unavailable on {host}: Core has no usage feed"
        ))
    } else {
        None
    };
    Response::Host {
        reply: Reply::Usage { host, usage, note },
    }
}

pub(super) fn usage_info(reading: &crate::usage::Reading, now: SystemTime) -> UsageInfo {
    let report = reading.report.as_ref();
    UsageInfo {
        provider: reading.provider.name().to_owned(),
        account: report.and_then(|report| {
            report
                .account
                .email
                .as_ref()
                .map(crate::usage::text::english)
        }),
        plan: report.and_then(|report| {
            report
                .account
                .plan
                .as_ref()
                .map(crate::usage::text::english)
        }),
        windows: report
            .map(|report| {
                report
                    .windows
                    .iter()
                    .map(|window| UsageWindow {
                        label: crate::usage::text::english_title(&window.kind),
                        used_percent: window.used,
                        resets_in_seconds: window.resets_in(now).map(|left| left.as_secs()),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        error: reading.error.as_ref().map(ToString::to_string),
    }
}

/// The answer to a request that opened a URL with the system handler.
/// `None` is a page that opens later, through a tunnel still connecting.
pub(super) fn page_answer(how: Option<PageOpened>, done: impl FnOnce() -> String) -> Response {
    match how {
        Some(PageOpened::SystemBrowser) => Response::done(done()),
        None => Response::done(format!("{}, once its tunnel connects", done())),
        Some(PageOpened::Unreachable) => {
            Response::error(ErrorCode::Failed, "This window does not show that host")
        }
    }
}
