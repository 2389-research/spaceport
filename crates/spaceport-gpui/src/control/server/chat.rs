//! `chat status`, `chat show`, `chat hide`, and `chat toggle`, answered at
//! once, and `chat new`, which waits for its runtime to acknowledge the new
//! tab and pane. When that outcome is uncertain the caller is told so and the
//! request is never sent again: the chat may exist.
use crate::control::protocol::{
    Call, Caller, ErrorCode, Response,
    chat::{CREATION_WAIT_SECONDS, ChatOp, Reply, Tab},
};
use gpui::App;
use std::{sync::mpsc::TryRecvError, time::Duration};

/// A start submitted to a runtime, waiting for its outcome.
enum Pending {
    Herdr(crate::chat::Receipt),
    Core(crate::core_terminals::Receipt),
}

impl Pending {
    /// The reply once the outcome is in; `None` keeps waiting.
    fn poll(&self, agent: &str, window_id: u64) -> Option<Response> {
        let result = match self {
            Self::Herdr(receipt) => match receipt.try_recv() {
                Ok(Ok(created)) => Ok((
                    Tab::Herdr {
                        tab_id: created.tab_id,
                    },
                    created.pane_id,
                )),
                Ok(Err(error)) => return Some(creation_error(&error)),
                Err(error) => Err(error),
            },
            Self::Core(receipt) => match receipt.try_recv() {
                Ok(Ok(created)) => {
                    let pane = crate::chat::core_pane_key(&created.terminal);
                    Ok((
                        Tab::Core {
                            tab_id: created.tab.into(),
                            terminal_id: created.terminal.to_string(),
                        },
                        pane,
                    ))
                }
                Ok(Err(error)) => return Some(core_creation_error(&error)),
                Err(error) => Err(error),
            },
        };
        match result {
            Ok((tab, pane_id)) => Some(Response::Chat {
                reply: Reply::Created {
                    agent: agent.into(),
                    window_id,
                    tab,
                    pane_id,
                },
            }),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(unknown(ErrorCode::Failed)),
        }
    }
}

/// The reply when the chat may or may not exist.
fn unknown(code: ErrorCode) -> Response {
    Response::error(
        code,
        "Chat creation outcome is unknown; inspect the window before retrying",
    )
}

/// A Herdr start that failed after it was submitted. Losing the answer, an
/// answer without the tab and pane, or a transport failure leaves the
/// outcome unknown; only Herdr's own refusal proves no chat exists.
fn creation_error(error: &crate::Error) -> Response {
    use crate::chat::{Error, StartError};
    // A refusal names the request the daemon did not run; anything else that
    // follows a submitted start may still have made the tab.
    let refused = |client: &spaceport_client::Error| {
        matches!(client, spaceport_client::Error::Endpoint { .. })
    };
    let known = || Response::error(ErrorCode::Failed, error.to_string());
    match error {
        crate::Error::Chat(Error::Start(StartError::Timeout)) => unknown(ErrorCode::Timeout),
        crate::Error::Chat(Error::Start(StartError::Response))
        | crate::Error::StaleConnection
        | crate::Error::NotConnected => unknown(ErrorCode::Failed),
        crate::Error::Chat(Error::Start(StartError::Request(inner))) => match &**inner {
            crate::Error::Client(client) if refused(client) => known(),
            _ => unknown(ErrorCode::Failed),
        },
        crate::Error::Client(client) if refused(client) => known(),
        crate::Error::Client(_) => unknown(ErrorCode::Failed),
        _ => known(),
    }
}

/// A Core start that failed after it was submitted.
fn core_creation_error(error: &crate::core_terminals::Error) -> Response {
    use spaceport_core_client::Error;
    match error {
        crate::core_terminals::Error::Core(Error::Timeout(_)) => unknown(ErrorCode::Timeout),
        // An invalid run reply or terminal ID follows submission, just like a lost answer.
        crate::core_terminals::Error::Core(
            Error::Lost | Error::Invalid(..) | Error::Backend(_),
        ) => unknown(ErrorCode::Failed),
        _ => Response::error(ErrorCode::Failed, error.to_string()),
    }
}

pub(super) fn serve(
    call: Call<ChatOp>,
    incoming: super::socket::Incoming,
    watches: &mut super::Watches,
    cx: &mut App,
) {
    let ChatOp::New { agent, pane } = call.op else {
        return incoming.respond(immediate(call, cx));
    };
    let handle = match super::window_for(&call.caller, cx) {
        Ok(handle) => handle,
        Err(response) => return incoming.respond(response),
    };
    let pending = match start(&call.caller, &agent, pane, handle, cx) {
        Ok(pending) => pending,
        Err(response) => return incoming.respond(response),
    };
    watches.add(
        incoming,
        Duration::from_secs(CREATION_WAIT_SECONDS),
        move |cx| {
            if handle.read_with(cx, |_, _| ()).is_err() {
                return Some(unknown(ErrorCode::NoWindow));
            }
            pending.poll(&agent, handle.window_id().as_u64())
        },
        unknown(ErrorCode::Timeout),
        cx,
    );
}

/// Submits the chat's start in `handle`'s window.
fn start(
    caller: &Caller,
    agent: &str,
    pane: Option<String>,
    handle: gpui::WindowHandle<crate::SpaceportWindow>,
    cx: &mut App,
) -> Result<Pending, Response> {
    let agent = agent
        .parse::<spaceport_chat_protocol::Agent>()
        .map_err(|_| {
            Response::error(
                ErrorCode::InvalidRequest,
                format!(
                    "Unknown agent {agent}; known agents are {}",
                    spaceport_chat_protocol::Agent::names()
                ),
            )
        })?;
    let beside = pane.or_else(|| caller.pane_id.clone());
    handle
        .update(cx, |view, _, cx| {
            // With no Herdr pane named, as from an agent in a herdr-core
            // pane, a herdr-core tab in the active group splits instead.
            let core = beside
                .is_none()
                .then(|| view.core_chat_group_tab(None, cx))
                .flatten();
            let started = match core {
                Some(tab) => view.start_core_chat(agent, tab, cx).map(Pending::Core),
                None if beside.is_none() && view.selected_core_group().is_some() => {
                    Err(crate::chat::StartError::CoreTerminalRequired)
                }
                None => view
                    .start_chat(agent, beside.as_deref())
                    .map(Pending::Herdr),
            };
            cx.notify();
            started.map_err(|error| Response::error(start_code(&error), error.to_string()))
        })
        .unwrap_or_else(|_| Err(Response::error(ErrorCode::NoWindow, "The window closed")))
}

/// Answers the chat requests that do not wait.
pub(super) fn immediate(call: Call<ChatOp>, cx: &mut App) -> Response {
    use super::{in_window, not_found};

    let Call { caller, op } = call;
    match op {
        ChatOp::Status => in_window(&caller, cx, |view, _, _| Response::Chat {
            reply: Reply::Chats {
                chats: info::infos(view),
            },
        }),
        ChatOp::New { .. } => Response::error(
            ErrorCode::InvalidRequest,
            "Chat creation requires an acknowledgement wait",
        ),
        ChatOp::View { switch, pane } => in_window(&caller, cx, |view, window, cx| {
            // A herdr-core pane is nobody's Herdr pane: the one holding the
            // keyboard comes before Herdr's focused pane.
            let pane = pane
                .or_else(|| caller.pane_id.clone())
                .or_else(|| view.focused_core_chat_key(window, cx))
                .or_else(|| {
                    view.live
                        .snapshot
                        .as_ref()
                        .and_then(|snapshot| snapshot.focused_pane_id.clone())
                });
            let Some(pane) = pane else {
                return Response::error(ErrorCode::InvalidRequest, "Name a pane with --pane");
            };
            if !view.chats.pane(&pane).is_some_and(|chat| chat.available()) {
                return not_found(format!("Pane {pane} has no chat this window can show"));
            }
            let shown = switch.apply(view.chats.shown(&pane));
            view.set_chat_raw(&pane, !shown, window, cx);
            Response::Chat {
                reply: Reply::View {
                    pane_id: pane,
                    shown,
                },
            }
        }),
    }
}

/// How a chat that did not start is reported to the caller.
fn start_code(error: &crate::chat::StartError) -> ErrorCode {
    use crate::chat::StartError;
    match error {
        StartError::AgentMissing(_)
        | StartError::CoreTerminalRequired
        | StartError::Disabled
        | StartError::NotLocal
        | StartError::ProgramMissing => ErrorCode::Unsupported,
        StartError::Environment(_) => ErrorCode::InvalidRequest,
        StartError::NoWorkspace => ErrorCode::NotFound,
        StartError::SlotBusy | StartError::ProgramUnknown => ErrorCode::Busy,
        StartError::NotConnected
        | StartError::Request(_)
        | StartError::Response
        | StartError::Timeout => ErrorCode::Failed,
    }
}

mod info {
    use crate::{
        SpaceportWindow,
        chat::{ChatInput, Source},
        control::protocol::chat::{ChatInfo, InputMode},
    };
    use spaceport_chat_protocol::Status;

    pub(super) fn infos(view: &SpaceportWindow) -> Vec<ChatInfo> {
        view.chats
            .panes()
            .map(|chat| {
                let transcript = chat.session.transcript();
                let mut status = match transcript.status {
                    Status::Starting => "starting",
                    Status::Idle => "idle",
                    Status::Working => "working",
                    Status::Failed { .. } => "failed",
                    Status::Exited { .. } => "exited",
                };
                if matches!(chat.target.source, Source::Observed(_))
                    && view.live.snapshot.as_ref().is_some_and(|snapshot| {
                        snapshot.agents.iter().any(|agent| {
                            agent.pane_id == chat.target.pane_id
                                && agent.agent_status
                                    == spaceport_client::protocol::AgentStatus::Blocked
                        })
                    })
                {
                    status = "blocked";
                }
                ChatInfo {
                    pane_id: chat.target.pane_id.clone(),
                    agent: chat.session.agent().to_owned(),
                    input_mode: match chat.input() {
                        ChatInput::Composer => InputMode::Chat,
                        ChatInput::ReadOnly => InputMode::ReadOnly,
                        ChatInput::Terminal => InputMode::Terminal,
                    },
                    session_id: transcript
                        .session
                        .as_ref()
                        .map(|session| session.id.clone()),
                    status: status.into(),
                    attached: chat.attached(),
                    shown: view.chats.shown(&chat.target.pane_id),
                    permissions: transcript.permissions.len(),
                    queued: transcript.queued.len(),
                    model: transcript
                        .option(spaceport_chat_protocol::OptionCategory::Model)
                        .map(spaceport_chat_protocol::ConfigOption::current_name),
                    context: transcript
                        .usage
                        .as_ref()
                        .filter(|usage| usage.size > 0)
                        .map(spaceport_chat_protocol::Usage::percent),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
