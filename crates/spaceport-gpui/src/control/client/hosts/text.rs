//! How host answers read in a terminal. Pure, so the layout is tested
//! without an app; `finish` makes the app's text safe to print.
use crate::control::protocol::{app::HostInfo, hosts::Ports};
use std::fmt::Write as _;

pub(super) fn hosts(hosts: &[HostInfo]) -> String {
    if hosts.is_empty() {
        return "The window has no hosts".into();
    }
    hosts
        .iter()
        .map(|host| {
            format!(
                "{} {} ({}): {}{}{}",
                if host.selected { ">" } else { " " },
                host.label,
                host.id,
                host.status,
                if host.muted { ", muted" } else { "" },
                if host.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", host.tags.join(", "))
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn ports(ports: &Ports) -> String {
    let Ports {
        host,
        workspace,
        listening,
        forwards,
        note,
    } = ports;
    let mut text = String::new();
    if listening.is_empty() {
        let _ = writeln!(text, "Workspace {workspace} on {host} listens on no port");
    } else {
        let _ = writeln!(text, "Workspace {workspace} on {host} listens on:");
    }
    for port in listening {
        let _ = write!(text, "  {:<24}", port.bind);
        if let Some(process) = &port.process {
            let _ = write!(text, " {process}");
        }
        if let Some(url) = &port.local_url {
            let _ = write!(text, "  {url}");
        }
        text.push('\n');
    }
    if !forwards.is_empty() {
        text.push_str("Forwarded:\n");
    }
    for forward in forwards {
        let _ = write!(
            text,
            "  {}:{} {}",
            forward.host, forward.port, forward.state
        );
        if let Some(url) = &forward.local_url {
            let _ = write!(text, "  {url}");
        }
        if let Some(reason) = &forward.reason {
            let _ = write!(text, " ({reason})");
        }
        text.push('\n');
    }
    if let Some(note) = note {
        text.push_str(note);
    }
    text.trim_end().to_owned()
}

/// `1.5 GiB`, in binary units as the status bar shows memory.
pub(in crate::control::client) fn bytes(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = "B";
    for next in ["KiB", "MiB", "GiB", "TiB"] {
        if value < 1024. {
            break;
        }
        value /= 1024.;
        unit = next;
    }
    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}
