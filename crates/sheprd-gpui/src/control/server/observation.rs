// ABOUTME: Brokers bounded read-only observations from prepared connected window state.
// ABOUTME: Fences opaque sources to exact connection generations, boots, panes, and chats.
mod content;
#[cfg(test)]
mod tests;
use crate::{
    SheprdWindow,
    control::protocol::{Call, Response, observation::*},
    state::LiveState,
};
#[cfg(test)]
use content::excerpt;
use gpui::{App, Global};
use sheprd_client::{ConnectTarget, protocol::ClientShellSnapshot};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex, Weak},
    time::{SystemTime, UNIX_EPOCH},
};

const KEEP: usize = 4096;
const MAX_COVERAGE: usize = 64;
const LABEL_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Identity {
    window: u64,
    endpoint: String,
    generation: u64,
    inbox: usize,
    boot: String,
    pane: String,
    workspace: String,
    tab: String,
    chat: Option<crate::chat::Target>,
    chat_session: Option<String>,
    chat_epoch: Option<uuid::Uuid>,
}
#[derive(Clone)]
struct Record {
    identity: Identity,
    inbox: Weak<Mutex<LiveState>>,
    daemon: ConnectTarget,
    source: Source,
}
#[derive(Default)]
struct Registry {
    records: HashMap<Target, Record>,
    keys: HashMap<Identity, Target>,
    order: VecDeque<Target>,
}
struct Projection {
    revision: u64,
    witnesses: Vec<(u64, usize)>,
}
struct ObservedPane<'a> {
    pane_id: &'a str,
    workspace_id: &'a str,
    tab_id: &'a str,
    label: &'a str,
    agent_status: sheprd_client::protocol::AgentStatus,
    agent: bool,
}

fn panes(snapshot: &ClientShellSnapshot) -> impl Iterator<Item = ObservedPane<'_>> {
    let known: HashSet<_> = snapshot
        .agents
        .iter()
        .map(|agent| agent.pane_id.as_str())
        .collect();
    let agents = snapshot.agents.iter().map(|agent| ObservedPane {
        pane_id: &agent.pane_id,
        workspace_id: &agent.workspace_id,
        tab_id: &agent.tab_id,
        label: agent
            .terminal_title_stripped
            .as_deref()
            .or(agent.title.as_deref())
            .or(agent.name.as_deref())
            .unwrap_or("Agent"),
        agent_status: agent.agent_status,
        agent: true,
    });
    let terminals = snapshot
        .panes
        .iter()
        .filter(move |pane| !known.contains(pane.pane_id.as_str()))
        .map(|pane| ObservedPane {
            pane_id: &pane.pane_id,
            workspace_id: &pane.workspace_id,
            tab_id: &pane.tab_id,
            label: pane.label.as_deref().unwrap_or("Terminal"),
            agent_status: sheprd_client::protocol::AgentStatus::Unknown,
            agent: false,
        });
    agents.chain(terminals)
}
impl Global for Registry {}
impl Registry {
    fn issue(&mut self, mut record: Record) -> Source {
        if let Some(target) = self.keys.get(&record.identity) {
            record.source.target = target.clone();
            self.records.insert(target.clone(), record.clone());
            return record.source;
        }
        while self.records.len() >= KEEP {
            if let Some(target) = self.order.pop_front() {
                if let Some(old) = self.records.remove(&target) {
                    self.keys.remove(&old.identity);
                }
            } else {
                break;
            }
        }
        let target = Target(uuid::Uuid::new_v4().to_string());
        record.source.target = target.clone();
        self.keys.insert(record.identity.clone(), target.clone());
        self.records.insert(target.clone(), record.clone());
        self.order.push_back(target);
        record.source
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| time.as_millis().min(u128::from(u64::MAX)) as u64)
}
fn label(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .scan(0, |bytes, c| {
            *bytes += c.len_utf8();
            (*bytes <= LABEL_BYTES).then_some(c)
        })
        .collect()
}
fn unavailable(target: Option<Target>, reason: UnavailableReason) -> Response {
    Response::Observation {
        reply: Reply::Unavailable { target, reason },
    }
}
fn invalid(error: Error, target: Option<Target>) -> Response {
    unavailable(
        target,
        match error {
            Error::InvalidBounds => UnavailableReason::InvalidBounds,
            Error::InvalidTarget => UnavailableReason::InvalidTarget,
        },
    )
}
fn live(view: &SheprdWindow, index: usize) -> &LiveState {
    if index == view.selected_endpoint {
        &view.live
    } else {
        &view.endpoints[index].live
    }
}
fn cover(coverage: &mut Vec<Coverage>, truncated: &mut bool, item: Coverage) {
    if coverage.len() < MAX_COVERAGE {
        coverage.push(item);
    } else {
        *truncated = true;
    }
}
fn snapshot(
    view: &SheprdWindow,
    record: &Record,
) -> Result<(usize, Arc<ClientShellSnapshot>), UnavailableReason> {
    let identity = &record.identity;
    let index = view
        .endpoints
        .iter()
        .position(|endpoint| endpoint.id == identity.endpoint)
        .ok_or(UnavailableReason::StaleTarget)?;
    let endpoint = &view.endpoints[index];
    if endpoint.generation != identity.generation
        || endpoint.connection.target != record.daemon
        || !Weak::ptr_eq(&record.inbox, &Arc::downgrade(&endpoint.connection.inbox))
    {
        return Err(UnavailableReason::StaleTarget);
    }
    let state = live(view, index);
    if !endpoint.enabled || !state.status.is_connected() {
        return Err(UnavailableReason::Disconnected);
    }
    let snapshot = state
        .snapshot
        .clone()
        .ok_or(UnavailableReason::NotObserved)?;
    if snapshot.boot_id != identity.boot {
        return Err(UnavailableReason::StaleTarget);
    }
    if identity.chat.is_some() {
        content::transcript(view, record, index)?;
    }
    let matches = panes(&snapshot)
        .filter(|agent| {
            agent.pane_id == identity.pane
                && agent.workspace_id == identity.workspace
                && agent.tab_id == identity.tab
        })
        .count();
    match matches {
        0 => Err(UnavailableReason::StaleTarget),
        1 => Ok((index, snapshot)),
        _ => Err(UnavailableReason::AmbiguousTarget),
    }
}

pub(super) fn serve(call: Call<Operation>, cx: &mut App) -> Response {
    match call.op {
        Operation::Sessions {
            offset,
            limit,
            status,
        } => sessions(offset, limit, status, cx),
        Operation::Conversation { target, bounds } => {
            read(target, bounds, ExcerptKind::Conversation, cx)
        }
        Operation::Terminal { target, bounds } => read(target, bounds, ExcerptKind::Terminal, cx),
    }
}

fn sessions(
    offset: usize,
    limit: usize,
    filter: Option<sheprd_client::protocol::AgentStatus>,
    cx: &mut App,
) -> Response {
    if offset > MAX_OFFSET || limit == 0 {
        return invalid(Error::InvalidBounds, None);
    }
    if cx.try_global::<Registry>().is_none() {
        cx.set_global(Registry::default());
    }
    let mut windows = super::main_windows(cx);
    windows.sort_by_key(|window| window.window_id().as_u64());
    let mut rows = Vec::<Session>::new();
    let mut records = Vec::<Record>::new();
    let mut sessions = HashMap::<(usize, String, String), usize>::new();
    let mut coverage = Vec::new();
    let mut coverage_truncated = false;
    // Reserve visible coverage for Core without inspecting any Core state.
    cover(
        &mut coverage,
        &mut coverage_truncated,
        Coverage::Runtime {
            name: "Herdr Core".into(),
            unavailable: if crate::core_feature::enabled(cx) {
                UnavailableReason::Unsupported
            } else {
                UnavailableReason::Disabled
            },
        },
    );
    let mut daemons = Vec::<ConnectTarget>::new();
    let mut projections = HashMap::<(usize, String), Projection>::new();
    let mut witness_count = 0;
    let mut capacity = false;
    // Pick one coherent freshest projection before applying status filters.
    for window in &windows {
        let id = window.window_id().as_u64();
        let _ = window.read_with(cx, |view, _| {
            for (index, endpoint) in view.endpoints.iter().enumerate() {
                let state = live(view, index);
                let observed = state
                    .snapshot
                    .as_ref()
                    .filter(|_| endpoint.enabled && state.status.is_connected());
                cover(
                    &mut coverage,
                    &mut coverage_truncated,
                    Coverage::Connection {
                        window: id,
                        host: label(&endpoint.label),
                        unavailable: if !endpoint.enabled {
                            Some(UnavailableReason::Disabled)
                        } else if !state.status.is_connected() {
                            Some(UnavailableReason::Disconnected)
                        } else if observed.is_none() {
                            Some(UnavailableReason::NotObserved)
                        } else {
                            None
                        },
                    },
                );
                let Some(snapshot) = observed else {
                    continue;
                };
                if witness_count >= KEEP {
                    capacity = true;
                    continue;
                }
                witness_count += 1;
                let daemon = daemons
                    .iter()
                    .position(|target| *target == endpoint.connection.target)
                    .unwrap_or_else(|| {
                        daemons.push(endpoint.connection.target.clone());
                        daemons.len() - 1
                    });
                let key = (daemon, snapshot.boot_id.clone());
                match projections.get_mut(&key) {
                    Some(current) if current.revision > snapshot.revision => {}
                    Some(current) if current.revision == snapshot.revision => {
                        current.witnesses.push((id, index))
                    }
                    Some(current) => {
                        *current = Projection {
                            revision: snapshot.revision,
                            witnesses: vec![(id, index)],
                        }
                    }
                    None => {
                        projections.insert(
                            key,
                            Projection {
                                revision: snapshot.revision,
                                witnesses: vec![(id, index)],
                            },
                        );
                    }
                }
            }
        });
    }
    let mut selected: Vec<_> = projections
        .into_iter()
        .flat_map(|((daemon, boot), projection)| {
            projection
                .witnesses
                .into_iter()
                .map(move |(window, index)| (daemon, boot.clone(), window, index))
        })
        .collect();
    selected.sort_by_key(|(_, _, window, index)| (*window, *index));
    for (daemon, boot, id, index) in selected {
        let Some(window) = windows
            .iter()
            .find(|window| window.window_id().as_u64() == id)
        else {
            continue;
        };
        let _ = window.read_with(cx, |view, _| {
            let endpoint = &view.endpoints[index];
            let state = live(view, index);
            let Some(snapshot) = state.snapshot.as_ref() else {
                return;
            };
            for agent in panes(snapshot) {
                if [
                    agent.pane_id,
                    agent.workspace_id,
                    agent.tab_id,
                    snapshot.boot_id.as_str(),
                ]
                .iter()
                .any(|value| value.is_empty())
                {
                    cover(
                        &mut coverage,
                        &mut coverage_truncated,
                        Coverage::Connection {
                            window: id,
                            host: label(&endpoint.label),
                            unavailable: Some(UnavailableReason::Unsupported),
                        },
                    );
                    continue;
                }
                let key = (daemon, boot.clone(), agent.pane_id.to_owned());
                if rows.len() >= KEEP && !sessions.contains_key(&key) {
                    capacity = true;
                    continue;
                }
                if filter.is_some_and(|status| status != agent.agent_status) {
                    continue;
                }
                let chat = (agent.agent && index == view.selected_endpoint)
                    .then(|| view.chats.pane(agent.pane_id))
                    .flatten()
                    .filter(|chat| {
                        chat.target.surface == crate::chat::Surface::Herdr && chat.attached()
                    });
                let source = Source {
                    target: Target(String::new()),
                    window: id,
                    host: label(&endpoint.label),
                    workspace: label(
                        snapshot
                            .workspaces
                            .iter()
                            .find(|workspace| workspace.workspace_id == agent.workspace_id)
                            .map_or("", |workspace| workspace.label.as_str()),
                    ),
                    label: label(agent.label),
                };
                let terminal = state.surface.as_ref().is_some_and(|surface| {
                    surface.boot_id == snapshot.boot_id
                        && surface.projection_revision == snapshot.revision
                        && surface
                            .panes
                            .iter()
                            .any(|pane| pane.pane_id == agent.pane_id)
                });
                let row = Session {
                    source: source.clone(),
                    status: agent.agent_status,
                    conversation: chat.is_some(),
                    terminal,
                    snapshot_revision: snapshot.revision,
                };
                let chat_target = chat.map(|chat| chat.target.clone());
                let chat_session = chat.and_then(|chat| {
                    chat.session
                        .transcript()
                        .session
                        .as_ref()
                        .map(|session| session.id.clone())
                });
                let record = Record {
                    identity: Identity {
                        window: id,
                        endpoint: endpoint.id.clone(),
                        generation: endpoint.generation,
                        inbox: Arc::as_ptr(&endpoint.connection.inbox) as usize,
                        boot: snapshot.boot_id.clone(),
                        pane: agent.pane_id.to_owned(),
                        workspace: agent.workspace_id.to_owned(),
                        tab: agent.tab_id.to_owned(),
                        chat: chat_target,
                        chat_session,
                        chat_epoch: chat
                            .and_then(|chat| chat.session.since().map(|cursor| cursor.epoch)),
                    },
                    inbox: Arc::downgrade(&endpoint.connection.inbox),
                    daemon: endpoint.connection.target.clone(),
                    source,
                };
                if let Some(position) = sessions.get(&key).copied() {
                    if (row.conversation, row.terminal)
                        > (rows[position].conversation, rows[position].terminal)
                    {
                        rows[position] = row;
                        records[position] = record;
                    }
                } else {
                    sessions.insert(key, rows.len());
                    rows.push(row);
                    records.push(record);
                }
            }
        });
    }
    if capacity {
        cover(&mut coverage, &mut coverage_truncated, Coverage::Capacity);
    }
    let total = rows.len();
    let end = offset.saturating_add(limit.min(MAX_SESSIONS)).min(total);
    let mut page = Vec::new();
    let registry = cx.global_mut::<Registry>();
    for (mut row, record) in rows
        .into_iter()
        .zip(records)
        .skip(offset)
        .take(limit.min(MAX_SESSIONS))
    {
        row.source = registry.issue(record);
        page.push(row);
    }
    Response::Observation {
        reply: Reply::Sessions {
            sessions: page,
            coverage,
            coverage_truncated,
            next_offset: (end < total).then_some(end),
            observed_at_ms: now_ms(),
        },
    }
}

fn record(target: &Target, cx: &App) -> Result<Record, UnavailableReason> {
    cx.try_global::<Registry>()
        .and_then(|registry| registry.records.get(target))
        .cloned()
        .ok_or(UnavailableReason::UnknownTarget)
}
fn read(target: Target, bounds: Bounds, kind: ExcerptKind, cx: &mut App) -> Response {
    if let Err(error) = target.validate() {
        return invalid(error, Some(target));
    }
    let bounds = match bounds.validated() {
        Ok(bounds) => bounds,
        Err(error) => return invalid(error, Some(target)),
    };
    let record = match record(&target, cx) {
        Ok(record) => record,
        Err(reason) => return unavailable(Some(target), reason),
    };
    let Some(window) = super::main_windows(cx)
        .into_iter()
        .find(|window| window.window_id().as_u64() == record.identity.window)
    else {
        return unavailable(Some(target), UnavailableReason::WindowClosed);
    };
    window
        .read_with(cx, |view, _| {
            let (index, snapshot) = match snapshot(view, &record) {
                Ok(value) => value,
                Err(reason) => return unavailable(Some(target.clone()), reason),
            };
            let result = match kind {
                ExcerptKind::Conversation => content::conversation(view, &record, index, bounds),
                ExcerptKind::Terminal => {
                    content::terminal(live(view, index), &snapshot, &record.identity.pane, bounds)
                }
            };
            match result {
                Ok(text) => Response::Observation {
                    reply: Reply::Excerpt {
                        source: Box::new(record.source.clone()),
                        kind,
                        text: text.text,
                        offset: bounds.offset,
                        next_offset: text.next_offset,
                        truncated: text.truncated,
                        observed_at_ms: now_ms(),
                        snapshot_revision: snapshot.revision,
                    },
                },
                Err(reason) => unavailable(Some(target.clone()), reason),
            }
        })
        .unwrap_or_else(|_| unavailable(Some(target), UnavailableReason::WindowClosed))
}

pub(crate) fn open_source(target: &Target, cx: &mut App) -> Response {
    if let Err(error) = target.validate() {
        return invalid(error, Some(target.clone()));
    }
    let record = match record(target, cx) {
        Ok(record) => record,
        Err(reason) => return unavailable(Some(target.clone()), reason),
    };
    let Some(window) = super::main_windows(cx)
        .into_iter()
        .find(|window| window.window_id().as_u64() == record.identity.window)
    else {
        return unavailable(Some(target.clone()), UnavailableReason::WindowClosed);
    };
    window
        .update(cx, |view, window, cx| {
            if let Err(reason) = snapshot(view, &record) {
                return unavailable(Some(target.clone()), reason);
            }
            if view.menu.page.is_some() {
                return Response::error(
                    crate::control::ErrorCode::WindowBusy,
                    "Close the window dialog before opening this source",
                );
            }
            if !view.select_endpoint(&record.identity.endpoint, cx) || !view.input_ready() {
                return Response::error(
                    crate::control::ErrorCode::Busy,
                    "Observed session is not ready for exact navigation; try again",
                );
            }
            if let Err(reason) = snapshot(view, &record) {
                return unavailable(Some(target.clone()), reason);
            }
            if !view.navigate(
                crate::NavigationTarget::Pane(record.identity.pane.as_str()),
                cx,
            ) {
                return Response::error(
                    crate::control::ErrorCode::Busy,
                    "Could not queue exact source navigation",
                );
            }
            view.show_activity_panel(crate::sidebar::ActivityPanel::Workspaces, window, cx);
            view.activity.return_to = Some(crate::sidebar::ActivityPanel::Ask);
            view.terminal_focus_moved(&record.identity.tab, false, cx);
            window.focus(&view.focus, cx);
            window.activate_window();
            cx.notify();
            Response::done("Requested observed session navigation")
        })
        .unwrap_or_else(|_| unavailable(Some(target.clone()), UnavailableReason::WindowClosed))
}
