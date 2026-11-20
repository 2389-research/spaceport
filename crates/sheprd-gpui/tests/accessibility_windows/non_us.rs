// ABOUTME: Qualifies native AltGr translation against real app shortcuts in disposable Windows CI.
// ABOUTME: Preserves the private runtime and requires exact layout and pressed-key cleanup receipts.
use super::{Duration, Running, fs};
use sheprd_client::{ClientEvent, ConnectOptions, ConnectTarget, protocol::ClientShellSnapshot};
use std::collections::BTreeSet;
use std::sync::Arc;

#[path = "non_us/matrix.rs"]
mod matrix;

fn snapshot(running: &Running) -> Arc<ClientShellSnapshot> {
    let client = sheprd_client::connect_with_surface_active(
        ConnectTarget::Socket(running.sandbox.socket().to_owned()),
        ConnectOptions::default(),
        false,
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match client
            .events
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .unwrap()
        {
            ClientEvent::Snapshot(snapshot) => return snapshot,
            ClientEvent::Disconnected { reason, .. } => {
                panic!("private observer disconnected: {reason}")
            }
            _ => {}
        }
    }
}

fn identities(
    snapshot: &ClientShellSnapshot,
) -> (String, BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    (
        snapshot.boot_id.clone(),
        snapshot
            .workspaces
            .iter()
            .map(|workspace| workspace.workspace_id.clone())
            .collect(),
        snapshot.tabs.iter().map(|tab| tab.tab_id.clone()).collect(),
        snapshot
            .panes
            .iter()
            .map(|pane| pane.pane_id.clone())
            .collect(),
    )
}

#[test]
#[ignore = "requires disposable GitHub-hosted Windows desktop and HERDR_TEST_BINARY; enables then removes only its Polish profile"]
fn native_altgr_text_survives_the_workspace_shortcut_and_restores_layouts() {
    for (name, expected) in [
        ("GITHUB_ACTIONS", "true"),
        ("RUNNER_ENVIRONMENT", "github-hosted"),
        ("RUNNER_OS", "Windows"),
    ] {
        assert_eq!(
            std::env::var(name).as_deref(),
            Ok(expected),
            "non-US session setup requires a disposable GitHub-hosted Windows runner"
        );
    }
    let mut running = Running::start();
    running.wait_tree("private native search field", |tree| {
        tree.contains("Edit 'Search spaces'") && tree.contains("TabItem '1' [selected=True]")
    });
    // The shared native key helper obtains foreground permission and enters chrome mode.
    running.read("^{F6}");
    let before = identities(&snapshot(&running));
    assert!(!before.1.is_empty() && !before.2.is_empty() && !before.3.is_empty());
    for (name, content) in [
        // keep-sorted start
        (
            "native_altgr.cs",
            include_str!("../support/native_altgr.cs"),
        ),
        (
            "native_profile_id.cs",
            include_str!("../support/native_profile_id.cs"),
        ),
        (
            "native_profile_id_tests.cs",
            include_str!("../support/native_profile_id_tests.cs"),
        ),
        ("uia_altgr.ps1", include_str!("../support/uia_altgr.ps1")),
        // keep-sorted end
    ] {
        fs::write(running.sandbox.dir.join(name), content).unwrap();
    }
    let capture = running.sandbox.dir.join("native-altgr.log");
    let mut command = running.powershell(
        &running.sandbox.dir.join("uia_altgr.ps1"),
        "native-altgr.log",
    );
    command.env("SPACEPORT_DISPOSABLE_LAYOUT_TEST", "github-hosted-windows");
    let limit = Duration::from_secs(40);
    let deadline = (std::time::SystemTime::now() + limit)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    command.args(["-DeadlineEpochMs", &deadline]);
    let output = running.command_output(command, &capture, limit).unwrap();
    assert!(output.contains("ALTGR native PASS: klid=00000415 text=U+0142,U+0144,U+017C,U+0143 workspace_dialog=false keys_released=true layouts_restored=true failure_cleanup=true"), "{output}");
    assert_eq!(identities(&snapshot(&running)), before);
    assert!(
        running
            .daemon
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none()
    );
    eprintln!("{output}");
}
