// ABOUTME: Verifies queued Core spawns survive an in-flight request.
// ABOUTME: Uses private scripted sockets to exercise window placement and queue draining.

#![cfg(unix)]
use super::super::{Purpose, spawning::Spawning};
use crate::{SheprdWindow, browser::Scope, sidebar};
use gpui::{Entity, TestAppContext, VisualTestContext};
use serde_json::{Value, json};
use sheprd_backend::Spawn;
use sheprd_core_client::frame::{self, Kind};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

fn purpose(name: &str) -> Purpose {
    Purpose::Tab {
        scope: Scope::endpoint("test"),
        workspace: name.into(),
    }
}

fn request(peer: &mut UnixStream) -> Value {
    loop {
        let mut header = [0; frame::HEADER];
        peer.read_exact(&mut header).unwrap();
        let len = u32::from_le_bytes(header[5..9].try_into().unwrap()) as usize;
        assert!(len <= frame::MAX_PAYLOAD);
        let mut payload = vec![0; len];
        peer.read_exact(&mut payload).unwrap();
        if header[..5] == [0; 5] {
            return serde_json::from_slice(&payload).unwrap();
        }
    }
}

fn answer(peer: &mut UnixStream, request: &Value, result: Value) {
    let reply = json!({"jsonrpc":"2.0", "id":request["id"], "result":result});
    peer.write_all(&frame::encode(0, Kind::Data, &serde_json::to_vec(&reply).unwrap()).unwrap())
        .unwrap();
}

fn refuse(peer: &mut UnixStream, request: &Value) {
    let reply = json!({"jsonrpc":"2.0", "id":request["id"], "error":{"code":-1,"message":"fixture refusal"}});
    peer.write_all(&frame::encode(0, Kind::Data, &serde_json::to_vec(&reply).unwrap()).unwrap())
        .unwrap();
}

fn place_until(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext, name: Option<&str>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let done = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.place_started_core_terminal(window, cx);
                match (&view.core_terminals.spawning, name) {
                    (Spawning::Idle, None) => true,
                    (
                        Spawning::Running {
                            purpose: Purpose::Tab { workspace, .. },
                            ..
                        },
                        Some(name),
                    ) => workspace == name,
                    _ => false,
                }
            })
        });
        if done {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "placement never drained to {name:?}"
        );
        std::thread::yield_now();
    }
}

#[gpui::test]
fn requests_during_a_spawn_keep_fifo_order(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            for name in ["first", "second", "third"] {
                assert!(
                    view.start_core_program(
                        purpose(name),
                        Spawn {
                            argv: vec![name.into()],
                            machine: Some("lab".into()),
                            ..Spawn::default()
                        },
                        cx
                    )
                    .is_ok()
                );
            }
        })
    });
    for name in ["first", "second", "third"] {
        let called = request(&mut peer);
        assert_eq!(called["method"], "terminal.run");
        assert_eq!(called["params"]["argv"], json!([name]));
        assert_eq!(called["params"]["on"], "lab");
        refuse(&mut peer, &called);
        let next = match name {
            "first" => Some("second"),
            "second" => Some("third"),
            _ => None,
        };
        place_until(&view, cx, next);
    }
}

#[gpui::test]
fn placement_drains_the_next_spawn_without_a_hub_change(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(
                view.start_core_program(purpose("first"), Spawn::default(), cx)
                    .is_ok()
            );
            assert!(
                view.start_core_program(purpose("second"), Spawn::default(), cx)
                    .is_ok()
            );
        })
    });
    let called = request(&mut peer);
    answer(
        &mut peer,
        &called,
        json!({"address":"t1", "labels":{}, "argv":["sh"], "cwd":"/tmp", "state":"live", "created_ms":1, "screen":"", "cols":80, "rows":24}),
    );
    place_until(&view, cx, Some("second"));
    // Placement may also attach a view. Find the next spawn without answering those streams.
    loop {
        let called = request(&mut peer);
        if called["method"] == "terminal.run" {
            refuse(&mut peer, &called);
            break;
        }
        refuse(&mut peer, &called);
    }
    place_until(&view, cx, None);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            assert!(
                view.core_terminals
                    .infos
                    .get(&sheprd_backend::TerminalId::new("t1").unwrap())
                    .is_some()
            )
        })
    });
}

#[gpui::test]
fn a_split_queued_without_a_backend_keeps_the_restored_tabs_core(cx: &mut TestAppContext) {
    use super::super::{
        Hub,
        panes::{Axis, Panes},
    };
    use crate::browser::{CoreGroupKey, Location, Store};
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (other, _peer) = super::scripted_core::connect("core-b");
    cx.update(|_, cx| {
        let key =
            CoreGroupKey::new("core-a", None, "spaceport", uuid::Uuid::from_u128(71)).unwrap();
        let at = sheprd_backend::TerminalId::new("t1").unwrap();
        let tab = Store::update(cx, |store| {
            store.open_core(
                &key,
                Some(Location::CoreTerminal {
                    panes: Panes::single(at.clone()),
                    observe: false,
                }),
            )
        })
        .unwrap();
        view.update(cx, |view, cx| {
            view.core_terminals.hub = Hub::new(true);
            view.core_terminals
                .hub
                .set_join_for_test(|_| Err(super::super::Error::NotInstalled));
            assert!(
                view.start_core_program(
                    Purpose::Split {
                        tab,
                        at: at.clone(),
                        axis: Axis::Row
                    },
                    Spawn::default(),
                    cx
                )
                .is_ok()
            );
            view.core_terminals.hub.set_backend_for_test(other);
            assert!(
                view.start_core_shell(),
                "the restored tab must reject another Core before spawning"
            );
            assert!(matches!(view.core_terminals.spawning, Spawning::Idle));
            assert!(matches!(
                view.start_core_program(
                    Purpose::Split {
                        tab,
                        at,
                        axis: Axis::Row
                    },
                    Spawn::default(),
                    cx
                ),
                Err(crate::Error::StaleWorkspace)
            ));
        });
    });
}

fn fail_worker(
    _: sheprd_core_client::CoreBackend,
    _: Spawn,
    _: std::sync::mpsc::SyncSender<Result<sheprd_backend::TerminalInfo, super::super::Error>>,
) -> std::io::Result<()> {
    Err(std::io::Error::other("fixture worker creation failure"))
}

#[gpui::test]
fn a_worker_creation_failure_reaches_the_placement_warning_and_preserves_fifo(
    cx: &mut TestAppContext,
) {
    use super::super::{
        Hub,
        spawning::{Event, Outcome},
    };
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::scripted_core::connect("core-a");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub = Hub::new(true);
            view.core_terminals
                .hub
                .set_join_for_test(|_| Err(super::super::Error::NotInstalled));
            assert!(
                view.start_core_program(
                    purpose("first"),
                    Spawn {
                        argv: vec!["first".into()],
                        ..Spawn::default()
                    },
                    cx
                )
                .is_ok()
            );
            assert!(
                view.start_core_program(
                    purpose("second"),
                    Spawn {
                        argv: vec!["second".into()],
                        ..Spawn::default()
                    },
                    cx
                )
                .is_ok()
            );
            view.core_terminals
                .hub
                .set_backend_for_test(backend.clone());
            assert!(matches!(
                view.core_terminals.spawning.transition(Event::Start(
                    &backend,
                    &view.core_terminals.infos,
                    fail_worker
                )),
                Outcome::None
            ));
            view.place_started_core_terminal(window, cx);
            assert!(
                view.flash.as_ref().is_some_and(|(flash, _)| flash
                    .text
                    .contains("fixture worker creation failure")),
                "the failed worker must use the placement warning"
            );
        })
    });
    let called = request(&mut peer);
    assert_eq!(called["params"]["argv"], json!(["second"]));
    refuse(&mut peer, &called);
    place_until(&view, cx, None);
}

#[gpui::test]
fn an_unscoped_split_with_a_fresh_matching_owner_reaches_that_core(cx: &mut TestAppContext) {
    use super::super::{
        observation,
        panes::{Axis, Panes},
    };
    use sheprd_backend::{RunState, TerminalId, TerminalInfo};
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::scripted_core::connect("core-a");
    let at = TerminalId::new("t1").unwrap();
    let info = TerminalInfo {
        agent: sheprd_client::protocol::AgentStatus::Unknown,
        cwd: None,
        id: at.clone(),
        labels: Default::default(),
        name: None,
        state: RunState::Running,
        title: None,
    };
    cx.update(|_, cx| {
        observation::set_document_for_test_as(
            cx,
            "core-a",
            sheprd_core_client::wire::Doc {
                value: json!({"schema_version":1,"workspaces":[]}),
                version: 1,
            },
        );
        observation::set_terminals_for_test(cx, vec![info.clone()]);
        view.update(cx, |view, cx| {
            let tab =
                view.core_terminals
                    .show_for_test(Panes::single(at.clone()), false, vec![info], cx);
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(
                view.start_core_program(
                    Purpose::Split {
                        tab,
                        at,
                        axis: Axis::Row
                    },
                    Spawn::default(),
                    cx
                )
                .is_ok()
            );
        });
    });
    let called = request(&mut peer);
    assert_eq!(called["method"], "terminal.run");
    refuse(&mut peer, &called);
    place_until(&view, cx, None);
}
