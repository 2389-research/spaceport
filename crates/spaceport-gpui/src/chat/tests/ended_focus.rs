//! Where the keyboard goes when a chat's menu search held it and the
//! connection ends for good: the view stops drawing, so its composer must
//! not keep the keyboard.

use super::{
    core_panes,
    menus::settings,
    overlay::{attach_with, open, poll},
    *,
};
use crate::{
    chat::{chats::Menu, core_pane::key},
    search_input::SearchInput,
};
use gpui::{AppContext, Modifiers, TestAppContext};
use spaceport_chat_protocol::ByeReason;
use spaceport_client::ConnectTarget;

#[gpui::test]
fn an_ended_chat_gives_the_keyboard_back_to_the_window(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, settings(), cx);
    let model = cx.debug_bounds("chat-setting-model").unwrap();
    cx.simulate_click(model.center(), Modifiers::default());
    poll(&view, &program, cx);
    let searching = cx.update(|window, cx| {
        let pane = view.read(cx).chats.panes().next().unwrap();
        pane.menu_search_focused(window, cx)
    });
    assert!(searching, "the model search holds the keyboard");
    // Another protocol version is a permanent end: no retry, no view.
    send(
        &mut stream,
        &ServerMessage::Bye {
            reason: ByeReason::Version { supported: 0 },
        },
    );
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            !view.chats.panes().next().unwrap().available()
        })
    });
    let (composer, window_focus) = cx.update(|window, cx| {
        let view = view.read(cx);
        let pane = view.chats.panes().next().unwrap();
        let composer = pane
            .composer
            .as_ref()
            .is_some_and(|input| input.read(cx).focus.is_focused(window));
        (composer, view.focus.is_focused(window))
    });
    assert!(!composer, "the undrawn composer does not hold the keyboard");
    assert!(window_focus, "the window's own focus has the keyboard");
}

/// A herdr-core pane's chat ends for good while its menu search holds the
/// keyboard: the pane's terminal view takes it back, as it would with no
/// chat drawn over it.
#[gpui::test]
fn an_ended_core_chat_gives_the_keyboard_to_its_terminal(cx: &mut TestAppContext) {
    let (view, mut stream, program, cx) = core_panes::attached(cx);
    let t1 = core_panes::id("t1");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            // What a click on the chat runs: t1 becomes the pane in use.
            view.focus_chat(&key(&t1), window, cx);
            let driven = core_panes::tab(core_panes::DRIVEN, cx);
            view.draw_core_view_for_test(driven, t1.clone(), cx);
            let search = cx.new(SearchInput::new);
            let redraw = cx.observe(&search, |_, _, _| {});
            let focus = search.read(cx).focus.clone();
            let pane = view.chats.pane_mut(&key(&t1)).unwrap();
            pane.open_menu(Menu::Setting("model".into()), Some((search, redraw)));
            window.focus(&focus, cx);
        });
    });
    poll(&view, &program, cx);
    let searching = cx.update(|window, cx| {
        let pane = view.read(cx).chats.pane(&key(&t1)).unwrap();
        pane.menu_search_focused(window, cx)
    });
    assert!(searching, "the menu search holds the keyboard");
    send(
        &mut stream,
        &ServerMessage::Bye {
            reason: ByeReason::Version { supported: 0 },
        },
    );
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            !view.chats.pane(&key(&t1)).unwrap().available()
        })
    });
    let (composer, terminal) = cx.update(|window, cx| {
        let view = view.read(cx);
        let pane = view.chats.pane(&key(&t1)).unwrap();
        let composer = pane
            .composer
            .as_ref()
            .is_some_and(|input| input.read(cx).focus.is_focused(window));
        (composer, view.core_terminals.focused_pane(window, cx))
    });
    assert!(!composer, "the undrawn composer does not hold the keyboard");
    let driven = cx.update(|_, cx| core_panes::tab(core_panes::DRIVEN, cx));
    assert_eq!(terminal, Some((driven, t1)), "t1's terminal view has it");
}
