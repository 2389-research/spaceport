// ABOUTME: Presents bounded observations while retaining their typed JSON source identity.
// ABOUTME: Reports unavailable and partial evidence without implying unseen sessions are idle.
use super::{Context, Output, exit, unexpected};
use crate::control::protocol::{
    Request, Response,
    observation::{Coverage, Operation, Reply, UnavailableReason},
};
pub(super) fn run(op: Operation, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Observation, op)? {
        Response::Observation { reply } => output(reply),
        response => Err(unexpected(response)),
    }
}
pub(super) fn output(reply: Reply) -> crate::Result<Output> {
    let text = match &reply {
        Reply::Sessions {
            sessions,
            coverage,
            coverage_truncated,
            next_offset,
            ..
        } => {
            let mut lines: Vec<String> = sessions
                .iter()
                .map(|session| {
                    format!(
                        "{}  {:?}  {} / {} / {}",
                        session.source.target.0,
                        session.status,
                        session.source.host,
                        session.source.workspace,
                        session.source.label
                    )
                })
                .collect();
            if sessions.is_empty() {
                lines.push("No observed sessions".into());
            }
            for item in coverage {
                match item {
                    Coverage::Connection {
                        host,
                        unavailable: Some(reason),
                        ..
                    } => lines.push(format!("Connection {host}: {reason:?}")),
                    Coverage::Runtime { name, unavailable } => {
                        lines.push(format!("Runtime {name}: {unavailable:?}"))
                    }
                    Coverage::Capacity => lines.push("Observation capacity reached".into()),
                    Coverage::Connection {
                        unavailable: None, ..
                    } => {}
                }
            }
            if *coverage_truncated {
                lines.push("Coverage list truncated".into());
            }
            if let Some(offset) = next_offset {
                lines.push(format!("Next page: --offset {offset}"));
            }
            lines.join("\n")
        }
        Reply::Excerpt {
            text, truncated, ..
        } => format!(
            "{text}{}",
            if *truncated {
                "\n[Excerpt truncated]"
            } else {
                ""
            }
        ),
        Reply::Unavailable { reason, .. } => format!("Observation unavailable: {reason:?}"),
    };
    let mut output = Output::data(text, &reply)?;
    if let Reply::Unavailable { reason, .. } = reply {
        output.status = match reason {
            UnavailableReason::InvalidBounds | UnavailableReason::InvalidTarget => exit::USAGE,
            UnavailableReason::Disabled | UnavailableReason::Unsupported => exit::UNSUPPORTED,
            UnavailableReason::Capacity => exit::BUSY,
            UnavailableReason::Disconnected => exit::REFUSED,
            _ => exit::NOT_FOUND,
        };
    }
    Ok(output)
}
