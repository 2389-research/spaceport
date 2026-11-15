//! What the chat side reads of herdr-core tabs: the terminals a chat may be
//! drawn over, the pane a new chat splits, and moving the pane in use.

use crate::{
    browser::TabId,
    core_terminals::{CoreTerminals, panes::Panes},
};
use gpui::{App, TestAppContext};
use sheprd_backend::{RunState, TerminalId, TerminalInfo};
use sheprd_client::protocol::AgentStatus;

fn id(name: &str) -> TerminalId {
    TerminalId::new(name).unwrap()
}

fn info(name: &str) -> TerminalInfo {
    TerminalInfo {
        agent: AgentStatus::Unknown,
        cwd: None,
        id: id(name),
        labels: Default::default(),
        name: None,
        state: RunState::Running,
        title: None,
    }
}

/// The first tab drives t1 and t2; the second mirrors t3.
fn terminals(cx: &mut App) -> (CoreTerminals, TabId, TabId) {
    let mut terminals = CoreTerminals::default();
    let mut panes = Panes::single(id("t1"));
    panes
        .split(&id("t1"), crate::core_terminals::panes::Axis::Row, id("t2"))
        .unwrap();
    let infos = ["t1", "t2", "t3", "t4"].map(info).to_vec();
    let driven = terminals.show_for_test(panes, false, infos, cx);
    let mirror = terminals.show_for_test(Panes::single(id("t3")), true, Vec::new(), cx);
    (terminals, driven, mirror)
}

#[gpui::test]
fn chats_are_drawn_only_over_panes_of_tabs_that_drive_them(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let (terminals, driven, _) = terminals(cx);
        let mut shown: Vec<&str> = terminals
            .chat_terminals(cx)
            .map(|info| info.id.as_str())
            .collect();
        shown.sort_unstable();
        assert_eq!(shown, ["t1", "t2"]);
        assert_eq!(terminals.tab_of(&id("t2"), cx), Some(driven));
        assert_eq!(terminals.tab_of(&id("t3"), cx), None);
        assert_eq!(terminals.tab_of(&id("t4"), cx), None);
    });
}

#[gpui::test]
fn a_new_chat_splits_the_pane_in_use_but_never_a_mirror(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let (mut terminals, driven, mirror) = terminals(cx);
        assert_eq!(terminals.chat_split_at(driven, cx), Some(id("t1")));
        assert_eq!(terminals.chat_split_at(mirror, cx), None);
        assert_eq!(terminals.chat_split_at(TabId::test(9), cx), None);
        assert!(terminals.set_active(driven, id("t2"), cx));
        assert_eq!(terminals.chat_split_at(driven, cx), Some(id("t2")));
    });
}

#[gpui::test]
fn the_pane_in_use_moves_only_to_a_pane_of_its_tab(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let (mut terminals, driven, _) = terminals(cx);
        assert!(!terminals.set_active(driven, id("t1"), cx));
        assert!(!terminals.set_active(driven, id("t3"), cx));
        assert!(!terminals.set_active(TabId::test(9), id("t1"), cx));
        assert!(terminals.set_active(driven, id("t2"), cx));
    });
}
