// ABOUTME: Exercises tab admission and eventual placement through private Core sockets.
// ABOUTME: Ensures intervening window actions cannot orphan a successfully created terminal.
use super::*;
use crate::browser::{Location, Store};

#[gpui::test]
fn a_replaced_store_reports_the_created_terminal_without_replaying_it(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(
                view.start_core_program(purpose("reserved"), Spawn::default(), cx)
                    .is_ok()
            );
        })
    });
    let called = request(&mut peer);
    assert_eq!(called["method"], "terminal.run");
    cx.update(|_, cx| cx.set_global(Store::default()));
    answer(
        &mut peer,
        &called,
        json!({"address":"orphan1", "labels":{}, "argv":["sh"], "state":"live", "created_ms":1, "screen":"", "cols":80, "rows":24}),
    );
    place_until(&view, cx, None);
    view.read_with(cx, |view, cx| {
        let text = &view.flash.as_ref().unwrap().0.text;
        assert!(
            text.contains("orphan1") && text.contains("started, but its tab could not be restored"),
            "{text}"
        );
        assert!(
            view.core_terminals
                .infos
                .get(&sheprd_backend::TerminalId::new("orphan1").unwrap())
                .is_some()
        );
        assert!(cx.global::<Store>().tabs().next().is_none());
        assert!(matches!(view.core_terminals.spawning, Spawning::Idle));
    });
}

#[gpui::test]
fn a_runtime_refusal_releases_the_reserved_slot(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(
                view.start_core_program(purpose("reserved"), Spawn::default(), cx)
                    .is_ok()
            );
            Store::update(cx, |store| {
                while store
                    .open(Scope::endpoint("other"), "w", Some(Location::Devices), None)
                    .is_some()
                {}
            });
        })
    });
    let called = request(&mut peer);
    refuse(&mut peer, &called);
    place_until(&view, cx, None);
    cx.update(|_, cx| {
        Store::update(cx, |store| {
            assert!(
                store
                    .open(Scope::endpoint("after"), "w", Some(Location::Devices), None)
                    .is_some()
            )
        })
    });
}

#[gpui::test]
fn a_full_store_refuses_runtime_creation_without_queueing(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, _peer) = super::super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            Store::update(cx, |store| {
                while store
                    .open(Scope::endpoint("other"), "w", Some(Location::Devices), None)
                    .is_some()
                {}
            });
            view.core_terminals.hub.set_backend_for_test(backend);
            assert!(matches!(
                view.start_core_program(purpose("new"), Spawn::default(), cx),
                Err(crate::Error::Browser(crate::browser::Error::Capacity))
            ));
            assert!(matches!(view.core_terminals.spawning, Spawning::Idle));
        })
    });
}

#[gpui::test]
fn a_queued_tab_keeps_its_slot_when_other_opens_fill_the_store(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals
                .hub
                .set_join_for_test(|_| Err(super::super::super::Error::NotInstalled));
            assert!(
                view.start_core_program(purpose("reserved"), Spawn::default(), cx)
                    .is_ok()
            );
            Store::update(cx, |store| {
                while store
                    .open(Scope::endpoint("other"), "w", Some(Location::Devices), None)
                    .is_some()
                {}
            });
            view.core_terminals.hub.set_backend_for_test(backend);
            view.start_core_shell();
        })
    });
    let called = request(&mut peer);
    assert_eq!(called["method"], "terminal.run");
    answer(
        &mut peer,
        &called,
        json!({"address":"t1", "labels":{}, "argv":["sh"], "cwd":"/tmp", "state":"live", "created_ms":1, "screen":"", "cols":80, "rows":24}),
    );
    place_until(&view, cx, None);
    cx.update(|_, cx| {
        let store = cx.global::<Store>();
        let scope = Scope::endpoint("test");
        let placed = store.in_workspace(&scope, "reserved").collect::<Vec<_>>();
        assert_eq!(placed.len(), 1);
        assert!(matches!(&placed[0].location, Some(Location::CoreTerminal { panes, .. }) if panes.contains(&sheprd_backend::TerminalId::new("t1").unwrap())));
    });
}
