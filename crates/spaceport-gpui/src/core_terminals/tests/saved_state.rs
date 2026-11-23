// ABOUTME: Checks Core tab admission with unreadable saved layouts.
// ABOUTME: Checks that a saved tab can still send a split request to its own Core.
use super::super::{Start, State, spawning::Spawning, workspace_document};
use crate::{
    browser::{CoreGroupKey, Location, Store, Tab},
    controls::Command,
    sidebar::layout_tests::fixture_core_window,
    window::Flash,
};
use gpui::TestAppContext;
use spaceport_backend::TerminalId;
use spaceport_panes::Panes;
use std::sync::Arc;

fn install_saved_terminal(
    cx: &mut TestAppContext,
    layouts: &[u8],
) -> (tempfile::TempDir, Tab, TerminalId) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("browser-tabs.json");
    let key = CoreGroupKey::new(
        "core-a",
        None,
        workspace_document::NAME,
        uuid::Uuid::from_u128(71),
    )
    .unwrap();
    let (scope, workspace_id) = key.parts();
    let at = TerminalId::new("t1").unwrap();
    let location = Location::CoreTerminal {
        panes: Panes::single(at.clone()),
        observe: false,
    };
    let tab = Tab {
        id: 7_u64.into(),
        scope,
        workspace_id,
        title: location.default_title(),
        location: Some(location),
        origin: None,
    };
    let saved = serde_json::json!({"tabs": [&tab], "next": 8});
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    std::fs::write(dir.path().join("editor-groups.json"), layouts).unwrap();
    let store = Store::load(Some(path));
    cx.update(|cx| store.install(cx));
    (dir, tab, at)
}

#[gpui::test]
fn the_core_tab_command_warns_without_upgrading_a_join_when_layouts_are_unreadable(
    cx: &mut TestAppContext,
) {
    assert_core_tab_refused(
        cx,
        b"{",
        "Saved tabs or layouts need repair. Check browser-tabs.json and editor-groups.json, \
         then restart Spaceport before opening new tabs.",
    );
}

#[gpui::test]
fn the_core_tab_command_refuses_exhausted_saved_ids_without_upgrading_a_join(
    cx: &mut TestAppContext,
) {
    let layouts = serde_json::json!({
        "layouts": [{
            "scope": "local",
            "workspace": "w0",
            "layout": {
                "groups": [{"pick": {"kind": "page", "id": u64::MAX}, "share": 1}],
                "active": 0,
            },
        }],
    });
    assert_core_tab_refused(
        cx,
        &serde_json::to_vec(&layouts).unwrap(),
        "Saved tab IDs are exhausted. Check browser-tabs.json and editor-groups.json, \
         then restart Spaceport before opening new tabs.",
    );
}

fn assert_core_tab_refused(cx: &mut TestAppContext, layouts: &[u8], expected: &'static str) {
    let (_dir, saved, _) = install_saved_terminal(cx, layouts);
    let (view, visual) = cx.add_window_view(fixture_core_window);
    let (flash, join_unchanged, idle, restored) = visual.update(|window, cx| {
        view.update(cx, |view, cx| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).focused_workspace_id =
                Some("w0".into());
            // The fixture has no worker, so even an unexpected upgrade cannot start Core.
            view.core_terminals.hub.set_connecting_for_test();
            view.command(Command::NewCoreTerminal, window, cx);

            let join_unchanged = matches!(
                view.core_terminals.hub.state(),
                State::Connecting {
                    start: Start::Join,
                    upgrade: false,
                    ..
                }
            );
            (
                view.flash.as_ref().map(|(flash, _)| flash.clone()),
                join_unchanged,
                matches!(view.core_terminals.spawning, Spawning::Idle),
                cx.global::<Store>().get(saved.id).cloned(),
            )
        })
    });
    cx.quit();

    assert_eq!(flash, Some(Flash::warning(expected)));
    assert!(join_unchanged);
    assert!(idle);
    assert_eq!(restored, Some(saved));
}

#[cfg(unix)]
#[gpui::test]
fn unreadable_layouts_refuse_a_new_core_tab_before_sending_a_request(cx: &mut TestAppContext) {
    use super::super::Purpose;
    use spaceport_backend::Spawn;
    use spaceport_panes::Axis;

    let (_dir, saved, at) = install_saved_terminal(cx, b"{");
    let (view, visual) = cx.add_window_view(fixture_core_window);
    let (backend, mut peer) = super::scripted_core::connect("core-a");
    let (result, idle, ready, restored, split) = visual.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            let purpose = Purpose::Tab {
                scope: saved.scope.clone(),
                workspace: saved.workspace_id.clone(),
            };
            let result = view.start_core_program(purpose, Spawn::default(), cx);
            let idle = matches!(view.core_terminals.spawning, Spawning::Idle);
            let ready = view.core_terminals.hub.backend_for("core-a").is_ok();
            let restored = cx.global::<Store>().get(saved.id).cloned();
            let split = (result.is_err() && idle && ready && restored.as_ref() == Some(&saved))
                .then(|| {
                    let purpose = Purpose::Split {
                        tab: saved.id,
                        at,
                        axis: Axis::Row,
                    };
                    let spawn = Spawn {
                        argv: vec!["fixture-split".into()],
                        ..Spawn::default()
                    };
                    view.start_core_program(purpose, spawn, cx)
                });
            (result, idle, ready, restored, split)
        })
    });

    // The existing split supplies the first frame on the bounded fixture socket.
    let request = matches!(split, Some(Ok(_))).then(|| read_request(&mut peer));
    drop(peer);
    cx.quit();

    assert!(
        result.is_err(),
        "the new tab must be refused before spawning"
    );
    assert!(idle);
    assert!(ready);
    assert_eq!(restored, Some(saved));
    split.unwrap().unwrap();
    let request = request.unwrap().unwrap();
    assert_eq!(request["method"], "terminal.run");
    assert_eq!(
        request["params"]["argv"],
        serde_json::json!(["fixture-split"])
    );
}

#[cfg(unix)]
fn read_request(peer: &mut std::os::unix::net::UnixStream) -> std::io::Result<serde_json::Value> {
    use spaceport_core_client::frame;
    use std::io::{self, Read};

    let mut header = [0; frame::HEADER];
    peer.read_exact(&mut header)?;
    let len = u32::from_le_bytes(header[5..].try_into().unwrap()) as usize;
    if header[..5] != [0; 5] || len > frame::MAX_PAYLOAD {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected a control frame",
        ));
    }
    let mut payload = vec![0; len];
    peer.read_exact(&mut payload)?;
    serde_json::from_slice(&payload).map_err(io::Error::other)
}
