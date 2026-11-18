// ABOUTME: Requires daemon acknowledgement and native placement before a Core receipt succeeds.
// ABOUTME: Checks runtime refusal and abandoned receivers without replaying submitted creation.
use super::*;
use crate::browser::{Location, Store};
use std::sync::mpsc::TryRecvError;

#[gpui::test]
fn a_core_chat_receipt_names_the_existing_tab_and_new_split(cx: &mut TestAppContext) {
    use crate::{
        browser::CoreGroupKey,
        core_terminals::{TabPanes, panes::Panes},
    };
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    let (tab, receipt) = cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            view.config.features.acp_chats = true;
            view.chats.program = crate::chat::Program::Found("/fixture/sheprd-acp".into());
            let key =
                CoreGroupKey::new("hub-a", None, "spaceport", uuid::Uuid::from_u128(71)).unwrap();
            let at = sheprd_backend::TerminalId::new("original").unwrap();
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
            view.core_terminals.tabs.insert(tab, TabPanes::new(at));
            (
                tab,
                view.start_core_chat(sheprd_chat_protocol::Agent::Claude, tab, cx)
                    .unwrap(),
            )
        })
    });
    let called = request(&mut peer);
    assert_eq!(called["method"], "terminal.run");
    assert!(matches!(receipt.try_recv(), Err(TryRecvError::Empty)));
    answer(
        &mut peer,
        &called,
        json!({"address":"chat-receipt", "labels":{}, "argv":["sheprd-acp","claude"], "state":"live", "created_ms":1, "screen":"", "cols":80, "rows":24}),
    );
    place_until(&view, cx, None);
    let created = receipt.try_recv().unwrap().unwrap();
    assert_eq!(created.tab, tab);
    assert_eq!(created.terminal.as_str(), "chat-receipt");
    cx.update(|_, cx| assert!(matches!(&cx.global::<Store>().get(tab).unwrap().location, Some(Location::CoreTerminal {panes, ..}) if panes.contains(&created.terminal))));
}

#[gpui::test]
fn creation_receipt_waits_for_acknowledgement_and_placement(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    let receipt = cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            view.start_core_program(purpose("created"), Spawn::default(), cx)
                .unwrap()
        })
    });
    let called = request(&mut peer);
    assert_eq!(called["method"], "terminal.run");
    assert!(matches!(receipt.try_recv(), Err(TryRecvError::Empty)));
    answer(
        &mut peer,
        &called,
        json!({"address":"receipt1", "labels":{}, "argv":["sh"], "state":"live", "created_ms":1, "screen":"", "cols":80, "rows":24}),
    );
    // A daemon answer alone cannot claim that the native tab exists.
    assert!(matches!(receipt.try_recv(), Err(TryRecvError::Empty)));
    place_until(&view, cx, None);
    let created = receipt.try_recv().unwrap().unwrap();
    assert_eq!(created.terminal.as_str(), "receipt1");
    cx.update(|_, cx| {
        let tab = cx.global::<Store>().get(created.tab).unwrap();
        assert!(matches!(&tab.location, Some(Location::CoreTerminal {panes, ..}) if panes.contains(&created.terminal)));
    });
    assert!(matches!(
        receipt.try_recv(),
        Err(TryRecvError::Disconnected)
    ));
}

#[gpui::test]
fn runtime_refusal_keeps_its_error_and_dropped_receivers_do_not_retry(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    let (backend, mut peer) = super::super::scripted_core::connect("hub-a");
    let receipt = cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.core_terminals.hub.set_backend_for_test(backend);
            view.start_core_program(purpose("refused"), Spawn::default(), cx)
                .unwrap()
        })
    });
    let called = request(&mut peer);
    refuse(&mut peer, &called);
    place_until(&view, cx, None);
    assert!(matches!(
        receipt.try_recv().unwrap(),
        Err(super::super::super::Error::Core(_))
    ));
    let receipt = cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.start_core_program(purpose("abandoned"), Spawn::default(), cx)
                .unwrap()
        })
    });
    let called = request(&mut peer);
    drop(receipt);
    refuse(&mut peer, &called);
    place_until(&view, cx, None);
    view.read_with(cx, |view, _| {
        assert!(matches!(view.core_terminals.spawning, Spawning::Idle))
    });
    peer.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    assert!(
        matches!(peer.read(&mut [0]), Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut))
    );
}
