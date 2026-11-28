// ABOUTME: Checks pane actions and Back/Forward through two live SSH clients.
// ABOUTME: Runs only against a separate, explicitly selected expendable session.
use super::*;
use spaceport_mobile::MobilePaneAction;

#[test]
#[ignore = "requires SPACEPORT_TEST_PANE_ACTIONS_SESSION on the disposable SSH fixture"]
fn two_clients_observe_pane_edits_and_confirmed_history() {
    let host = SshHost {
        hostname: setting("SPACEPORT_TEST_SSH_HOST"),
        port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
        username: setting("SPACEPORT_TEST_SSH_USER"),
        fingerprint: Some(setting("SPACEPORT_TEST_SSH_FINGERPRINT")),
    };
    let open = || {
        MobileSession::open(
            host.clone(),
            SshAuth::Password {
                password: setting("SPACEPORT_TEST_SSH_PASSWORD"),
            },
            setting("SPACEPORT_TEST_PANE_ACTIONS_SESSION"),
            80,
            24,
            8,
            16,
        )
        .unwrap()
    };
    let phone = open();
    let view = wait(&phone, "phone focus", |v| v.input_target.is_some());
    let peer = open();
    wait(&peer, "second client's focus", |v| v.input_target.is_some());
    let boot = view.boot_id;
    let ws = view.focused_workspace.unwrap();
    let tab = view.focused_tab.unwrap();
    let first = view.input_target.unwrap();
    let eligible = view.pane_actions.unwrap();
    assert!(eligible.split && eligible.rename && eligible.zoom);
    let edit = |id: &str, action| {
        phone
            .apply_pane_action(boot.clone(), ws.clone(), tab.clone(), id.into(), action)
            .unwrap()
    };
    edit(&first.pane_id, MobilePaneAction::SplitRight);
    let split = wait(&phone, "focused right split", |v| {
        v.back_pane.as_ref() == Some(&first.pane_id) && v.input_target.is_some()
    });
    let right = split.focused_pane.unwrap();
    wait(&peer, "right split visible from the second client", |v| {
        v.focused_pane.as_ref() == Some(&right)
    });
    assert!(phone.send_text(first.clone(), "stale".into()).is_err());

    let back = phone
        .travel_to_pane(boot.clone(), first.pane_id.clone(), false)
        .unwrap();
    wait(&phone, "Back confirmed", |v| {
        v.go_to_arrival == Some(back) && v.forward_pane.as_ref() == Some(&right)
    });
    let next = phone
        .travel_to_pane(boot.clone(), right.clone(), true)
        .unwrap();
    wait(&phone, "Forward confirmed", |v| {
        v.go_to_arrival == Some(next) && v.input_target.is_some()
    });
    edit(
        &first.pane_id,
        MobilePaneAction::Rename {
            label: "Phone notes".into(),
        },
    );
    wait(&peer, "other client sees background pane rename", |v| {
        v.focused_pane.as_ref() == Some(&right)
            && v.panes
                .iter()
                .any(|p| p.id == first.pane_id && p.label == "Phone notes")
    });
    wait(&phone, "rename barrier", |v| v.input_target.is_some());
    for enabled in [true, false] {
        edit(&right, MobilePaneAction::Zoom { enabled });
        wait(&peer, "other client sees zoom state", |v| {
            v.tabs.iter().any(|t| t.id == tab && t.zoomed == enabled)
        });
        wait(&phone, "zoom barrier", |v| {
            v.input_target.is_some() && v.tabs.iter().any(|t| t.id == tab && t.zoomed == enabled)
        });
    }
    peer.focus_pane(boot.clone(), first.pane_id.clone())
        .unwrap();
    wait(&phone, "other client changed focus", |v| {
        v.focused_pane.as_ref() == Some(&first.pane_id) && v.back_pane.as_ref() == Some(&right)
    });
    edit(&first.pane_id, MobilePaneAction::SplitDown);
    wait(&peer, "other client sees down split", |v| {
        v.panes.len() == view.panes.len() + 2
    });
    phone.disconnect();
    peer.disconnect();
}
