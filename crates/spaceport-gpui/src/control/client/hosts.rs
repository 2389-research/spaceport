//! `host`, `ports`, and `usage`.
mod text;

pub(super) use text::bytes;

#[cfg(test)]
mod tests;

use super::{Context, Output, unexpected};
use crate::control::protocol::{
    Request, Response,
    hosts::{HostOp, Reply},
};

/// Prints what `spaceport host` answers.
pub(super) fn host(op: HostOp, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Host, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::Host {
            reply: Reply::Hosts { hosts },
        } => Output::data(text::hosts(&hosts), &hosts),
        response => Err(unexpected(response)),
    }
}

/// Prints what `spaceport ports` answers. Its operations are the host
/// family's too, so the word the user typed picks the printing.
pub(super) fn ports(op: HostOp, context: &Context) -> crate::Result<Output> {
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
