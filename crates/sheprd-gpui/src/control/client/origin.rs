// ABOUTME: Guards control commands from runtime-owned Core terminal environments.
// ABOUTME: Refuses inferred legacy targets without starting or querying a runtime.
use crate::control::protocol::{
    ErrorCode, Request,
    app::WindowOp,
    chat::ChatOp,
    events::{Condition, EventsOp},
    hosts::HostOp,
    tabs::TabOp,
    work::WorkOp,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Shell,
    Core,
}

impl Origin {
    pub(super) fn current() -> Self {
        if ["HC_TERMINAL_ID", "HC_TERMINAL_SOCKET"]
            .iter()
            .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
        {
            Self::Core
        } else {
            Self::Shell
        }
    }

    pub(super) fn check(self, request: &Request) -> crate::Result<()> {
        if self == Self::Shell {
            return Ok(());
        }
        let unsupported = match request {
            Request::Work(call) => !matches!(call.op, WorkOp::Notes),
            Request::Host(call) => matches!(
                call.op,
                HostOp::Ports
                    | HostOp::Forward { .. }
                    | HostOp::Unforward { .. }
                    | HostOp::OpenPort { .. }
                    | HostOp::Mute { host: None, .. }
            ),
            Request::Tab(call) => matches!(call.op, TabOp::Devices),
            Request::Chat(call) => matches!(
                call.op,
                ChatOp::New { .. } | ChatOp::View { pane: None, .. }
            ),
            Request::Events(call) => matches!(
                call.op,
                EventsOp::Wait {
                    condition: Condition::Online,
                    ..
                }
            ),
            _ => false,
        };
        if unsupported {
            return Err(Self::unsupported());
        }
        if matches!(
            request,
            Request::Status(_)
                | Request::Window(crate::control::protocol::Call {
                    op: WindowOp::List,
                    ..
                })
                | Request::Work(crate::control::protocol::Call {
                    op: WorkOp::Notes,
                    ..
                })
                | Request::Events(crate::control::protocol::Call {
                    op: EventsOp::Poll { .. }
                        | EventsOp::Wait {
                            condition: Condition::Window,
                            ..
                        },
                    ..
                })
        ) {
            return Ok(());
        }
        if request.caller().window.is_none() {
            return Err(crate::control::Error::Rejected {
                code: ErrorCode::NoWindow,
                message: "Core pane callers must select --window; use `spaceport window list` to find its ID".into(),
            }.into());
        }
        Ok(())
    }

    pub(super) fn unsupported() -> crate::Error {
        crate::control::Error::Rejected {
            code: ErrorCode::Unsupported,
            message: "This command has no qualified Core terminal/workspace target; it cannot act on a legacy session for a Core pane".into(),
        }.into()
    }
}
