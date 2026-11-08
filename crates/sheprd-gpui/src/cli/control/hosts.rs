//! `pr`, `host`, `ports`, and `security`.
use super::{Args, CliError, unknown};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrCommand {
    Status,
    Checks,
    Comments,
    Open,
    Quota,
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecurityCommand {
    List,
    /// Show an alert (`42`) or a private report (`GHSA-…`) of `repository`
    /// in the section, beside its list.
    Open {
        repository: String,
        item: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortsCommand {
    List,
    Forward(u16),
    Unforward(u16),
    Open(u16),
}

pub(super) fn pr(args: &mut Args) -> Result<PrCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(PrCommand::Status);
    };
    Ok(match verb.to_str() {
        Some("status") => PrCommand::Status,
        Some("checks") => PrCommand::Checks,
        Some("comments") => PrCommand::Comments,
        Some("open") => PrCommand::Open,
        Some("quota") => PrCommand::Quota,
        _ => return Err(unknown("pr", verb)),
    })
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

pub(super) fn security(args: &mut Args) -> Result<SecurityCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(SecurityCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => SecurityCommand::List,
        Some("open") => SecurityCommand::Open {
            repository: args.required_text("REPOSITORY")?,
            item: args.required_text("ALERT|GHSA")?,
        },
        _ => return Err(unknown("security", verb)),
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
