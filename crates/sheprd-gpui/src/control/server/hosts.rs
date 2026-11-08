//! `pr`, `host`, `ports`, `usage`, `system`, `security`, and
//! what the
//! window gathers about its hosts, read from the state it already keeps.
//! Nothing here starts a lookup, a scan, or a sample of its own; a request
//! answers from the latest one, and says why when there is none.
mod mute;
mod ports;
mod pr;
mod security;

#[cfg(test)]
mod tests;

use super::{Watches, in_free_window, in_window, socket, target, window_for};
use crate::{
    SheprdWindow,
    browser::PageOpened,
    control::protocol::{
        Call, Caller, ErrorCode, Response,
        hosts::{HostOp, LoadInfo, Reply, UsageInfo, UsageWindow},
    },
};
use gpui::App;
use sheprd_client::protocol::{ClientShellSnapshot, ClientShellWorkspace};
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
        HostOp::Pr => in_window(caller, cx, |view, _, _| pr::status(view, caller)),
        HostOp::PrQuota => in_window(caller, cx, |view, _, _| pr::quota(view)),
        HostOp::PrComments => return pr::comments(call.caller, incoming, watches, cx),
        HostOp::PrOpen => in_window(caller, cx, |view, window, cx| {
            pr::open(view, caller, window, cx)
        }),
        HostOp::Ports => in_window(caller, cx, |view, _, _| ports::list(view, caller)),
        HostOp::Forward { port } => in_window(caller, cx, |view, _, cx| {
            ports::forward(view, caller, port, cx)
        }),
        HostOp::Unforward { port } => in_window(caller, cx, |view, _, cx| {
            ports::unforward(view, caller, port, cx)
        }),
        HostOp::OpenPort { port } => return ports::open(call.caller, port, incoming, watches, cx),
        HostOp::Usage => in_window(caller, cx, |view, _, _| usage(view, SystemTime::now())),
        HostOp::System => in_window(caller, cx, |view, _, _| system(view)),
        HostOp::Security => in_window(caller, cx, |view, _, _| security::alerts(view)),
        HostOp::SecurityOpen { repository, item } => in_window(caller, cx, |view, window, cx| {
            security::open(view, &repository, &item, window, cx)
        }),
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

    pub(super) fn selected(&self, view: &SheprdWindow) -> bool {
        self.index == view.selected_endpoint
    }

    fn snapshot<'a>(&self, view: &'a SheprdWindow) -> Option<&'a ClientShellSnapshot> {
        let live = if self.selected(view) {
            &view.live
        } else {
            &view.endpoints.get(self.index)?.live
        };
        live.snapshot.as_deref()
    }

    pub(super) fn workspace<'a>(&self, view: &'a SheprdWindow) -> Option<&'a ClientShellWorkspace> {
        self.snapshot(view)?
            .workspaces
            .iter()
            .find(|workspace| workspace.workspace_id == self.workspace)
    }

    pub(super) fn focused(&self, view: &SheprdWindow) -> bool {
        self.selected(view)
            && self
                .snapshot(view)
                .and_then(|snapshot| snapshot.focused_workspace_id.as_deref())
                == Some(self.workspace.as_str())
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
                format!("No host is named {name}; `sheprd host list` lists them"),
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
        .unwrap_or_else(|| format!("Sheprd could not switch to {label}"));
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
    } else if entry.is_none() {
        Some(format!("Sheprd has not read usage on {host} yet"))
    } else if usage.is_empty() {
        Some(format!("No AI plan is signed in on {host}"))
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

/// The selected host's latest load sample.
fn system(view: &SheprdWindow) -> Response {
    let label = selected_label(view);
    let host = view
        .endpoints
        .get(view.selected_endpoint)
        .and_then(|endpoint| crate::usage::Host::load_of(&endpoint.connection.target));
    let Some(host) = host else {
        return Response::Host {
            reply: Reply::System {
                load: None,
                note: Some(format!("Load is not sampled on {label}, a cloud machine")),
            },
        };
    };
    let reading = view.system_load.get(&host);
    let load = reading
        .and_then(crate::system_load::Reading::latest)
        .map(|sample| Box::new(load_info(&label, sample)));
    let note = match (&load, reading.and_then(crate::system_load::Reading::error)) {
        (_, Some(error)) => Some(error.to_owned()),
        (Some(_), None) => None,
        (None, None) => Some(
            "Sheprd samples load only while it shows it: in the status bar \
             (show_system_load), the Devices overview, or a host picker"
                .to_owned(),
        ),
    };
    Response::Host {
        reply: Reply::System { load, note },
    }
}

pub(super) fn load_info(host: &str, sample: &crate::system_load::Sample) -> LoadInfo {
    LoadInfo {
        host: host.to_owned(),
        cpu_percent: sample.cpu,
        memory_used: sample.memory.map(|memory| memory.used),
        memory_total: sample.memory.map(|memory| memory.total),
        cores: sample.cores,
        load: sample.load,
        uptime_seconds: sample.uptime,
        disk_available: sample.disk.map(|disk| disk.available),
        disk_total: sample.disk.map(|disk| disk.total),
        gpu_count: sample.gpu.map(|gpu| gpu.count),
        gpu_percent: sample.gpu.and_then(|gpu| gpu.busy),
        gpu_memory_used: sample
            .gpu
            .and_then(|gpu| gpu.memory.map(|memory| memory.used)),
        gpu_memory_total: sample
            .gpu
            .and_then(|gpu| gpu.memory.map(|memory| memory.total)),
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
