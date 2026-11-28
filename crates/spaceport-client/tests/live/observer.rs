// ABOUTME: Exercises passive discovery, forwarding and a real Herdr endpoint handshake.
// ABOUTME: Confirms observation reports the same boot and never creates a stopped session.

use super::*;
use std::{
    os::{
        fd::OwnedFd,
        unix::{fs::symlink, net::UnixStream},
    },
    process::Stdio,
};

struct Forwarder(Option<Child>);

impl Drop for Forwarder {
    fn drop(&mut self) {
        stop_children([&mut self.0]);
    }
}

fn observe(sandbox: &Sandbox, session: &str) -> Client {
    let mut shell = sandbox.command("/bin/sh", "observer.log");
    spaceport_client::observe_embedded_bridge(session, move |command, _| {
        let (client, peer) = UnixStream::pair()?;
        let child = shell
            .arg("-c")
            .arg(command)
            .stdin(Stdio::from(OwnedFd::from(peer.try_clone()?)))
            .stdout(Stdio::from(OwnedFd::from(peer)))
            .spawn()?;
        Ok((client, Forwarder(Some(child))))
    })
    .unwrap()
}

#[test]
#[ignore = "requires an explicitly selected disposable Herdr test binary"]
fn reads_running_roster_without_starting_a_missing_session() {
    let binary = daemon_binary();
    let mut daemon = Daemon {
        sandbox: Sandbox::new(),
        child: None,
    };
    let sandbox = &daemon.sandbox;
    let bin_dir = sandbox.dir.join(".local/bin");
    fs::create_dir_all(&bin_dir).unwrap();
    symlink(&binary, bin_dir.join("herdr")).unwrap();
    let sessions = sandbox.dir.join("config/herdr/sessions");
    symlink(
        sessions.join("observed/herdr-client.sock"),
        sandbox.socket(),
    )
    .unwrap();
    daemon.child = Some(
        sandbox
            .command(&binary, "daemon.log")
            .args(["--session", "observed", "server"])
            .spawn()
            .unwrap(),
    );
    sandbox.wait_for_daemon(daemon.child.as_mut().unwrap(), TIMEOUT);

    let active = Session::open(&daemon);
    let expected = active.snapshot.as_ref().unwrap();
    let passive = observe(sandbox, "observed");
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match passive.events.recv_deadline(deadline).unwrap() {
            ClientEvent::Snapshot(snapshot) => {
                assert_eq!(snapshot.boot_id, expected.boot_id);
                assert_eq!(snapshot.focused_pane_id, expected.focused_pane_id);
                break;
            }
            ClientEvent::Connected(_) => (),
            event => panic!("unexpected observer event {event:?}"),
        }
    }
    drop(passive);
    let missing = observe(sandbox, "missing");
    assert!(matches!(
        missing.events.recv_timeout(TIMEOUT),
        Ok(ClientEvent::Disconnected { .. })
    ));
    assert!(!sessions.join("missing/herdr-client.sock").exists());
}
