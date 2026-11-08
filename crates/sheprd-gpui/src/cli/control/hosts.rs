//! `host` and `ports`.
use super::{Args, CliError, unknown};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostCommand {
    List,
    Select(String),
    /// Mute or unmute a host by ID or label; absent, the caller's.
    Mute {
        host: Option<String>,
        muted: bool,
    },
    /// Replace a host's tags; none clears them.
    Tag {
        id: String,
        tags: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortsCommand {
    List,
    Forward(u16),
    Unforward(u16),
    Open(u16),
}

pub(super) fn host(args: &mut Args) -> Result<HostCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(HostCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => HostCommand::List,
        Some("select") => HostCommand::Select(args.required_text("HOST")?),
        Some(verb @ ("mute" | "unmute")) => HostCommand::Mute {
            host: args.optional_text()?,
            muted: verb == "mute",
        },
        Some("tag") => HostCommand::Tag {
            id: args.required_text("HOST")?,
            tags: std::iter::from_fn(|| args.optional_text().transpose())
                .collect::<Result<_, _>>()?,
        },
        _ => return Err(unknown("host", verb)),
    })
}

fn port(args: &mut Args) -> Result<u16, CliError> {
    args.required_text("PORT")?
        .parse()
        .ok()
        .filter(|port| *port > 0)
        .ok_or(CliError::InvalidValue("PORT"))
}

pub(super) fn ports(args: &mut Args) -> Result<PortsCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(PortsCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => PortsCommand::List,
        Some("forward") => PortsCommand::Forward(port(args)?),
        Some("unforward") => PortsCommand::Unforward(port(args)?),
        Some("open") => PortsCommand::Open(port(args)?),
        _ => return Err(unknown("ports", verb)),
    })
}
