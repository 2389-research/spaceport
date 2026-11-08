//! `host`, `ports`, `usage`, and `system`.
mod text;

pub(super) use text::bytes;

#[cfg(test)]
mod tests;

use super::{Context, Output, unexpected};
use crate::{
    cli::control::{HostCommand, PortsCommand},
    control::protocol::{
        Request, Response,
        hosts::{HostOp, Reply},
    },
};

pub(super) fn host(command: HostCommand, context: &Context) -> crate::Result<Output> {
    let op = match command {
        HostCommand::List => HostOp::List,
        HostCommand::Select(id) => HostOp::Select { id },
        HostCommand::Mute { host, muted } => HostOp::Mute { host, muted },
        HostCommand::Tag { id, tags } => HostOp::Tag { id, tags },
    };
    match context.call(Request::Host, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::Host {
            reply: Reply::Hosts { hosts },
        } => Output::data(text::hosts(&hosts), &hosts),
        response => Err(unexpected(response)),
    }
}

pub(super) fn ports(command: PortsCommand, context: &Context) -> crate::Result<Output> {
    let op = match command {
        PortsCommand::List => HostOp::Ports,
        PortsCommand::Forward(port) => HostOp::Forward { port },
        PortsCommand::Unforward(port) => HostOp::Unforward { port },
        PortsCommand::Open(port) => HostOp::OpenPort { port },
    };
    match context.call(Request::Host, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::Host {
            reply: Reply::Ports(ports),
        } => {
            let empty = ports.listening.is_empty() && ports.forwards.is_empty();
            let output = Output::data(text::ports(&ports), &ports)?;
            Ok(if empty { output.nothing() } else { output })
        }
        response => Err(unexpected(response)),
    }
}

pub(super) fn usage(context: &Context) -> crate::Result<Output> {
    match context.call(Request::Host, HostOp::Usage)? {
        Response::Host {
            reply: reply @ Reply::Usage { .. },
        } => {
            let empty = matches!(&reply, Reply::Usage { usage, .. } if usage.is_empty());
            let output = Output::data(text::usage(&reply), &reply)?;
            Ok(if empty { output.nothing() } else { output })
        }
        response => Err(unexpected(response)),
    }
}

pub(super) fn system(context: &Context) -> crate::Result<Output> {
    match context.call(Request::Host, HostOp::System)? {
        Response::Host {
            reply: reply @ Reply::System { .. },
        } => {
            let empty = matches!(&reply, Reply::System { load: None, .. });
            let output = Output::data(text::system(&reply), &reply)?;
            Ok(if empty { output.nothing() } else { output })
        }
        response => Err(unexpected(response)),
    }
}
