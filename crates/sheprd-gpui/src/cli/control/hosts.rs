//! `host` and `ports`.
use super::{Args, CliError, HostOp, unknown};

pub(super) const HOST_USAGE: &str = "spaceport host [list]\n    The window's hosts and their status. Herdr's `machine` command\n    manages saved hosts.\nspaceport host mute|unmute [HOST]\n    Silence a host's notifications as a whole, or let them through again,\n    by ID or label; absent, the caller's host. Works while it is offline.\nspaceport host select ID\n    Show that host in the window, as choosing it in the sidebar does.\nspaceport host tag ID [TAG...]\n    Replace the local tags that group that host in the sidebar; with no\n    tags, clear them.";

pub(super) fn host(args: &mut Args) -> Result<HostOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(HostOp::List);
    };
    Ok(match verb.to_str() {
        Some("list") => HostOp::List,
        Some("select") => HostOp::Select {
            id: args.required_text("HOST")?,
        },
        Some(verb @ ("mute" | "unmute")) => HostOp::Mute {
            host: args.optional_text()?,
            muted: verb == "mute",
        },
        Some("tag") => HostOp::Tag {
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

pub(super) fn ports(args: &mut Args) -> Result<HostOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(HostOp::Ports);
    };
    Ok(match verb.to_str() {
        Some("list") => HostOp::Ports,
        Some("forward") => HostOp::Forward { port: port(args)? },
        Some("unforward") => HostOp::Unforward { port: port(args)? },
        Some("open") => HostOp::OpenPort { port: port(args)? },
        _ => return Err(unknown("ports", verb)),
    })
}
