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

pub(super) const HOST_USAGE: &str = "spaceport host [list]\n    The window's hosts and their status. Herdr's `machine` command\n    manages saved hosts.\nspaceport host mute|unmute [HOST]\n    Silence a host's notifications as a whole, or let them through again,\n    by ID or label; absent, the caller's host. Works while it is offline.\nspaceport host select ID\n    Show that host in the window, as choosing it in the sidebar does.\nspaceport host tag ID [TAG...]\n    Replace the local tags that group that host in the sidebar; with no\n    tags, clear them.";

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

pub(super) const PORTS_USAGE: &str = "spaceport ports [list]\n    Ports the caller's workspace's processes listen on, and forwards.\n    Listening ports are known only while show_listening_ports is on.\nspaceport ports forward|unforward PORT\n    Forward a port of the workspace's SSH host to this machine, or stop.\nspaceport ports open PORT\n    Open localhost:PORT with the system URL handler, forwarding it first if remote.";

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
