// ABOUTME: Reorders private Herdr workspaces through focused native keyboard input.
// ABOUTME: Verifies real daemon snapshots retain every workspace and terminal identity.
use super::{Duration, Instant, Running};
use sheprd_client::{
    Client, ClientEvent, ConnectOptions, ConnectTarget, protocol::ClientShellSnapshot,
};
use std::sync::Arc;

const FIRST: &str = "spaceport-keyboard-first";
const SECOND: &str = "spaceport-keyboard-second";

fn snapshot(
    client: &Client,
    boot: Option<&str>,
    accept: impl Fn(&ClientShellSnapshot) -> bool,
) -> Arc<ClientShellSnapshot> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let event = client
            .events
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("the private daemon must acknowledge native workspace order");
        match event {
            ClientEvent::Snapshot(snapshot) => {
                if let Some(boot) = boot {
                    assert_eq!(snapshot.boot_id, boot, "the owned daemon restarted");
                }
                if accept(&snapshot) {
                    return snapshot;
                }
            }
            ClientEvent::Disconnected { reason, .. } => {
                panic!("private observer disconnected: {reason}")
            }
            ClientEvent::CommandRejected { reason, .. } => {
                panic!("private observer rejected: {reason}")
            }
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "native workspace order did not arrive"
        );
    }
}

fn order(snapshot: &ClientShellSnapshot) -> Vec<String> {
    snapshot
        .workspaces
        .iter()
        .map(|workspace| workspace.workspace_id.clone())
        .collect()
}

fn press(running: &mut Running, key: &str) {
    let capture = running.sandbox.dir.join("workspace-key.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_tree.ps1"),
        "workspace-key.log",
    );
    command.args(["-FocusWorkspace", FIRST, "-Keys", key]);
    let output = running
        .command_output(command, &capture, Duration::from_secs(20))
        .unwrap();
    assert!(
        output.contains("WORKSPACE_KEY focused=true released=true sent=true"),
        "{output}"
    );
}

#[test]
#[ignore = "requires active Windows desktop and HERDR_TEST_BINARY; reorders only its private workspaces"]
fn native_workspace_arrows_change_daemon_order_and_preserve_terminal_ids() {
    let mut running = Running::start();
    running.wait_tree("private initial workspace", |tree| {
        tree.contains("Document 'Terminal'")
    });
    for name in [FIRST, SECOND] {
        running.invoke("New workspace");
        running.wait_tree("private workspace dialog", |tree| {
            tree.contains("Button 'Create'")
        });
        running.read(&format!("{name}{{ENTER}}"));
        running.wait_tree("named private workspace", |tree| {
            tree.contains(&format!("ListItem '{name}'")) && tree.contains("Document 'Terminal'")
        });
    }
    let observer = sheprd_client::connect_with_surface_active(
        ConnectTarget::Socket(running.sandbox.socket().to_owned()),
        ConnectOptions::default(),
        false,
    )
    .unwrap();
    let before = snapshot(&observer, None, |snapshot| {
        [FIRST, SECOND].iter().all(|name| {
            snapshot
                .workspaces
                .iter()
                .any(|workspace| workspace.label == *name)
        })
    });
    let original = order(&before);
    let first = before
        .workspaces
        .iter()
        .position(|workspace| workspace.label == FIRST)
        .unwrap();
    let second = before
        .workspaces
        .iter()
        .position(|workspace| workspace.label == SECOND)
        .unwrap();
    assert_eq!(
        second,
        first + 1,
        "private named workspaces must start adjacent"
    );
    let mut moved = original.clone();
    moved.swap(first, second);
    let terminal_ids = |snapshot: &ClientShellSnapshot| {
        snapshot
            .panes
            .iter()
            .map(|pane| pane.pane_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let terminals = terminal_ids(&before);
    assert!(!terminals.is_empty());
    // Chrome rows become keyboard stops only after native region entry.
    let focus = running.read("^b{F6}|^b{F6}");
    assert!(
        focus
            .lines()
            .last()
            .is_some_and(|line| line.contains("TabItem 'Workspaces")),
        "native region entry did not reach the sidebar: {focus}"
    );
    for (key, expected) in [("%{DOWN}", moved), ("%{UP}", original)] {
        press(&mut running, key);
        let after = snapshot(&observer, Some(&before.boot_id), |snapshot| {
            order(snapshot) == expected
        });
        assert_eq!(
            terminal_ids(&after),
            terminals,
            "keyboard reorder replaced a terminal"
        );
        assert_eq!(after.workspaces.len(), before.workspaces.len());
        running.wait_tree("native workspace row order", |tree| {
            let first = tree
                .find(&format!("ListItem '{FIRST}'"))
                .unwrap_or(usize::MAX);
            let second = tree
                .find(&format!("ListItem '{SECOND}'"))
                .unwrap_or(usize::MAX);
            first != usize::MAX && second != usize::MAX && (first < second) == (key == "%{UP}")
        });
    }
    assert!(
        running
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none()
    );
    eprintln!(
        "WORKSPACE_REORDER native PASS: both arrow directions, daemon order, native row order, stable workspace and terminal IDs"
    );
}
