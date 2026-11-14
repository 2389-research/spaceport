// ABOUTME: Pins what each exact-Core check does with an absent, lost, other, or matching backend.
// ABOUTME: Uses a scripted private Core socket, so no installed runtime is dialed.
//! One row per site that asks "is the hub's backend this Core, and alive".
//! The sites differ on purpose in places; each row is that site's contract.

use super::*;
use crate::{
    SheprdWindow,
    browser::{CoreGroupKey, Location, Pick, Store, TabId},
    core_terminals::{Error as HubError, Hub, Start, panes::Panes, tests::scripted_core},
    i18n::fl,
};
use sheprd_backend::{RunState, TerminalId, TerminalInfo};
use sheprd_client::protocol::AgentStatus;
use sheprd_core_client::CoreBackend;
use std::os::unix::net::UnixStream;

const HUB: &str = "hub-a";
const OTHER: &str = "hub-b";
const GROUP: Uuid = Uuid::from_u128(71);
/// A terminal the group already holds.
const MEMBER: &str = "t7";
/// A terminal Core reports that the group does not hold.
const LOOSE: &str = "t8";

/// What the hub holds when a site asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Held {
    Absent,
    /// The right Core, its connection dropped.
    Lost,
    /// A live connection to another Core.
    Other,
    /// Another Core, its connection dropped.
    LostOther,
    Match,
}

const HELD: [Held; 5] = [
    Held::Absent,
    Held::Lost,
    Held::Other,
    Held::LostOther,
    Held::Match,
];

/// The scripted Core's ends of the live connections; dropping one loses it.
type Peers = Vec<UnixStream>;

fn never_join(_: Start) -> Result<CoreBackend, HubError> {
    Err(HubError::NotInstalled)
}

/// Gives the window a fresh hub holding `held`. The hub can never dial.
fn hold(view: &mut SheprdWindow, held: Held, peers: &mut Peers) {
    view.core_terminals.hub = Hub::new(true);
    view.core_terminals.hub.set_join_for_test(never_join);
    let (core_id, lost) = match held {
        Held::Absent => return,
        Held::Lost => (HUB, true),
        Held::Other => (OTHER, false),
        Held::LostOther => (OTHER, true),
        Held::Match => (HUB, false),
    };
    let (backend, peer) = scripted_core::connect(core_id);
    if lost {
        scripted_core::lose(&backend, peer);
    } else {
        peers.push(peer);
    }
    view.core_terminals.hub.set_backend_for_test(backend);
}

fn info(id: &str) -> TerminalInfo {
    TerminalInfo {
        agent: AgentStatus::Unknown,
        cwd: None,
        id: TerminalId::new(id).unwrap(),
        labels: Default::default(),
        name: None,
        state: RunState::Running,
        title: None,
    }
}

/// A fresh observation of `HUB`: one group holding `MEMBER`, two terminals.
fn observe(cx: &mut gpui::App) -> CoreGroupKey {
    observation::set_document_for_test_as(
        cx,
        HUB,
        Doc {
            value: serde_json::json!({"schema_version": 1, "workspaces": [{
                "id": GROUP, "label": "One",
                "members": [{"core_id": HUB, "terminal_id": MEMBER}]
            }]}),
            version: 3,
        },
    );
    observation::set_terminals_for_test(cx, vec![info(MEMBER), info(LOOSE)]);
    CoreGroupKey::new(HUB, None, "spaceport", GROUP).unwrap()
}

fn core_tab(key: &CoreGroupKey, cx: &mut gpui::App) -> TabId {
    let location = Location::CoreTerminal {
        panes: Panes::single(TerminalId::new(MEMBER).unwrap()),
        observe: false,
    };
    Store::update(cx, |store| store.open_core(key, Some(location))).unwrap()
}

fn flash(view: &SheprdWindow) -> Option<String> {
    view.flash.as_ref().map(|(flash, _)| flash.text.to_string())
}

/// Reading a group: an absent backend passes, because the observation is
/// shared by every window and a group is shown before its window joins.
#[gpui::test]
fn reading_a_group_passes_without_a_backend_but_not_with_a_lost_or_other_one(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        let key = observe(cx);
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_group_available(&key, cx)
            })
        });
        assert_eq!(seen, [true, false, false, false, true]);
    });
}

/// A new terminal in a selected group needs the live matching Core. A lost
/// or other backend is refused earlier, as a group that is not available.
#[gpui::test]
fn a_new_terminal_in_a_group_needs_the_live_matching_core(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        let key = observe(cx);
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_terminals.workspace_ui.selected_group = Some(key.clone());
                view.core_terminals.starting = None;
                view.core_terminals.wanted = None;
                view.flash = None;
                view.new_core_terminal(window, cx);
                let terminals = &view.core_terminals;
                if terminals.starting.is_some() || terminals.wanted.is_some() {
                    "started".to_owned()
                } else {
                    flash(view).unwrap()
                }
            })
        });
        let stale = fl!("core-workspace-stale");
        let unavailable = fl!("core-terminals-open-workspace");
        assert_eq!(
            seen,
            [
                stale,
                unavailable.clone(),
                unavailable.clone(),
                unavailable,
                "started".to_owned()
            ]
        );
    });
}

/// A chat splits a group's Core tab only on the live matching Core.
#[gpui::test]
fn a_chat_splits_a_group_tab_only_on_the_live_matching_core(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        let key = observe(cx);
        let tab = core_tab(&key, cx);
        let mut peers = Peers::new();
        let group = view.update(cx, |view, cx| {
            hold(view, Held::Absent, &mut peers);
            view.core_terminals.workspace_ui.selected_group = Some(key.clone());
            let group = view.ensure_layout().unwrap().active();
            view.browser
                .layouts
                .get_mut(&key.parts())
                .unwrap()
                .choose(group, Pick::Page(tab));
            view.sync_core_views(cx);
            group
        });
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_chat_group_tab(Some(group), cx) == Some(tab)
            })
        });
        assert_eq!(seen, [false, false, false, false, true]);
    });
}

/// Opening a member's terminal needs the live matching Core.
#[gpui::test]
fn opening_a_member_needs_the_live_matching_core(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|window, cx| {
        observe(cx);
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_terminals.workspace_ui.selected_group = None;
                view.open_core_workspace_member(GROUP, HUB, MEMBER, window, cx);
                view.selected_core_group().is_some()
            })
        });
        assert_eq!(seen, [false, false, false, false, true]);
    });
}

/// Adding a member compares only the Core's identity; a lost connection to
/// the right Core gets as far as the edit, which refuses it as not connected.
#[gpui::test]
fn adding_a_member_checks_identity_then_the_edit_checks_the_connection(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observe(cx);
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.flash = None;
                view.toggle_core_workspace_member(GROUP, HUB.into(), LOOSE.into(), true, cx);
                if view.core_terminals.workspace_ui.pending.is_some() {
                    "submitted".to_owned()
                } else {
                    flash(view).unwrap()
                }
            })
        });
        let stale = fl!("core-workspace-stale");
        assert_eq!(
            seen,
            [
                stale.clone(),
                fl!("core-workspace-connect"),
                stale.clone(),
                stale,
                "submitted".to_owned()
            ]
        );
    });
}

/// Attaching a selected group's panes: only the live matching Core gets a
/// view. A lost or other backend stops earlier, at the unavailable group.
#[gpui::test]
fn a_group_tab_attaches_only_to_the_live_matching_core(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        let key = observe(cx);
        core_tab(&key, cx);
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_terminals.workspace_ui.selected_group = Some(key.clone());
                view.core_terminals.views.clear();
                view.sync_core_views(cx);
                view.core_terminals.views.len()
            })
        });
        assert_eq!(seen, [0, 0, 0, 0, 1]);
    });
}

/// Attaching a plain workspace's Core tab names no Core, so any backend the
/// hub holds is used, a lost one included; the next tick drops those views.
#[gpui::test]
fn a_plain_workspace_tab_attaches_to_whatever_backend_the_hub_holds(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, cx| {
        observe(cx);
        let (scope, workspace) = view.update(cx, |view, _| {
            let mut snapshot = sidebar::layout_tests::snapshot(1);
            snapshot.focused_workspace_id = Some("w0".into());
            view.live.snapshot = Some(std::sync::Arc::new(snapshot));
            view.browser_key().unwrap()
        });
        let location = Location::CoreTerminal {
            panes: Panes::single(TerminalId::new(MEMBER).unwrap()),
            observe: false,
        };
        Store::update(cx, |store| {
            store.open(scope, &workspace, Some(location), None)
        })
        .unwrap();
        let mut peers = Peers::new();
        let seen = HELD.map(|held| {
            view.update(cx, |view, cx| {
                hold(view, held, &mut peers);
                view.core_terminals.views.clear();
                view.sync_core_views(cx);
                view.core_terminals.views.len()
            })
        });
        assert_eq!(seen, [0, 1, 1, 1, 1]);
    });
}
