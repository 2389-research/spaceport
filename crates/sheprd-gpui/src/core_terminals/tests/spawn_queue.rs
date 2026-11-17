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
        view.update(cx, |view, _| {
            view.core_terminals.hub.set_backend_for_test(backend);
            for name in ["first", "second", "third"] {
                assert!(view.start_core_program(
                    purpose(name),
                    Spawn {
                        argv: vec![name.into()],
                        machine: Some("lab".into()),
                        ..Spawn::default()
                    }
                ));
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
        view.update(cx, |view, _| {
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(view.start_core_program(purpose("first"), Spawn::default()));
            assert!(view.start_core_program(purpose("second"), Spawn::default()));
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
