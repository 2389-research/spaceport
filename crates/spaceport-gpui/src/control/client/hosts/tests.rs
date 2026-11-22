#![allow(clippy::unwrap_used)]
use super::text;

use crate::control::protocol::{
    app::HostInfo,
    hosts::{ForwardInfo, PortInfo, Ports, Reply, UsageInfo, UsageWindow},
};

#[test]
fn hosts_read_one_per_entry() {
    let hosts = [
        HostInfo {
            id: "local".into(),
            label: "Local".into(),
            status: "online".into(),
            selected: true,
            muted: false,
            tags: Vec::new(),
        },
        HostInfo {
            id: "ssh:devbox".into(),
            label: "Devbox".into(),
            status: "offline".into(),
            selected: false,
            muted: true,
            tags: vec!["work".into(), "lab".into()],
        },
    ];
    assert_eq!(
        text::hosts(&hosts),
        "> Local (local): online\n  Devbox (ssh:devbox): offline, muted [work, lab]"
    );
}

#[test]
fn ports_list_listeners_forwards_and_why_none() {
    let reply = Ports {
        host: "Devbox".into(),
        workspace: "w_1".into(),
        listening: vec![PortInfo {
            port: 3000,
            bind: "*:3000".into(),
            process: Some("node".into()),
            local_url: Some("http://devbox:3000/".into()),
        }],
        forwards: vec![ForwardInfo {
            host: "me@devbox".into(),
            port: 5173,
            state: "ended".into(),
            local_url: None,
            reason: Some("ssh exited".into()),
        }],
        note: None,
    };
    assert_eq!(
        text::ports(&reply),
        "Workspace w_1 on Devbox listens on:\n  \
         *:3000                   node  http://devbox:3000/\n\
         Forwarded:\n  me@devbox:5173 ended (ssh exited)"
    );
    let empty = Ports {
        host: "Local".into(),
        workspace: "w_1".into(),
        listening: Vec::new(),
        forwards: Vec::new(),
        note: Some("Scanning is off".into()),
    };
    assert_eq!(
        text::ports(&empty),
        "Workspace w_1 on Local listens on no port\nScanning is off"
    );
}

#[test]
fn usage_reads_in_plain_units() {
    let reply = Reply::Usage {
        host: "Local".into(),
        usage: vec![UsageInfo {
            provider: "Claude".into(),
            account: Some("me@example.com".into()),
            plan: Some("Max".into()),
            windows: vec![UsageWindow {
                label: "Session".into(),
                used_percent: 12.4,
                resets_in_seconds: Some(2 * 3600 + 53 * 60),
            }],
            error: None,
        }],
        note: None,
    };
    assert_eq!(
        text::usage(&reply),
        "Claude · Max · me@example.com\n  Session       12% used, resets in 2h 53m"
    );
}

#[test]
fn durations_and_sizes_keep_two_units_at_most() {
    assert_eq!(text::duration(59), "0m");
    assert_eq!(text::duration(5 * 60), "5m");
    assert_eq!(text::duration(3600 + 60), "1h 1m");
    assert_eq!(text::duration(2 * 86_400 + 3600 + 60), "2d 1h");
    assert_eq!(text::bytes(512), "512 B");
    assert_eq!(text::bytes(1536), "1.5 KiB");
    assert_eq!(text::bytes(3 << 30), "3.0 GiB");
}
