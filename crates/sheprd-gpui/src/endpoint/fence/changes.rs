// ABOUTME: Applies what selection, reconnect, reboot and removal do to a connection's identity.
// ABOUTME: Shared by the tests that pin each deferred action's fence.
//! Each change moves only the identity terms its event moves. The real
//! events also run `reset_selected`, which drops or cancels most deferred
//! actions outright; these leave the action in place so its fence answers.

use crate::SheprdWindow;
use gpui::Context;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Change {
    /// `select_endpoint` on the endpoint already selected: the real call.
    Reselect,
    /// Another endpoint was selected, then this one again: two selections
    /// moved the epoch on and nothing else.
    SwitchAndReturn,
    /// The endpoint's connection was replaced, with a new generation and
    /// inbox, and polling has not yet reset the window onto it.
    ReconnectUnseen,
    /// The same, once the window has reset onto the new connection.
    Reconnect,
    /// The daemon restarted: the window's snapshot names another boot.
    Reboot,
    /// The endpoint left the list: its slot now names another endpoint.
    Removed,
}

pub(crate) const CHANGES: [Change; 6] = [
    Change::Reselect,
    Change::SwitchAndReturn,
    Change::ReconnectUnseen,
    Change::Reconnect,
    Change::Reboot,
    Change::Removed,
];

/// Applies `change` to the endpoint at `index`, which the action came from.
#[allow(clippy::unwrap_used)]
pub(crate) fn apply(
    view: &mut SheprdWindow,
    index: usize,
    change: Change,
    cx: &mut Context<SheprdWindow>,
) {
    match change {
        Change::Reselect => {
            let id = view.endpoints[index].id.clone();
            assert!(view.select_endpoint(&id, cx));
        }
        Change::SwitchAndReturn => view.selection_epoch += 2,
        Change::ReconnectUnseen => replace_connection(view, index),
        Change::Reconnect => {
            replace_connection(view, index);
            view.selection_epoch += 1;
            view.selected_generation = view.endpoints[index].generation;
        }
        Change::Reboot => Arc::make_mut(view.live.snapshot.as_mut().unwrap())
            .boot_id
            .push_str("-rebooted"),
        Change::Removed => view.endpoints[index].id.push_str("-other"),
    }
}

/// A new generation and inbox, as `Endpoint::stop` leaves them. The handle
/// stays, so a test can see whether an action that passed its fence sent.
fn replace_connection(view: &mut SheprdWindow, index: usize) {
    let endpoint = &mut view.endpoints[index];
    endpoint.generation += 1;
    endpoint.connection.inbox = Arc::new(Mutex::new(endpoint.live.clone()));
}

/// A change as a click on a notice sees it. A notice names the connection
/// it came from, selected or not, so these also move one term at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClickChange {
    Shared(Change),
    /// Another endpoint is selected; the notice's own is untouched.
    AnotherSelected,
    /// Only the endpoint's generation moved on.
    GenerationOnly,
    /// Only the endpoint's inbox was replaced.
    InboxOnly,
}

pub(crate) const CLICK_CHANGES: [ClickChange; 9] = [
    ClickChange::Shared(Change::Reselect),
    ClickChange::AnotherSelected,
    ClickChange::Shared(Change::SwitchAndReturn),
    ClickChange::GenerationOnly,
    ClickChange::InboxOnly,
    ClickChange::Shared(Change::ReconnectUnseen),
    ClickChange::Shared(Change::Reconnect),
    ClickChange::Shared(Change::Reboot),
    ClickChange::Shared(Change::Removed),
];

/// What a click still reaches after each of `CLICK_CHANGES`, in order: the
/// connection the notice came from, for as long as that connection exists.
pub(crate) const CLICK_REACHES: [bool; 9] =
    [true, true, true, false, false, false, false, true, false];

/// Applies `change` to the endpoint at `index`, which the notice came from.
#[allow(clippy::unwrap_used)]
pub(crate) fn apply_click(
    view: &mut SheprdWindow,
    index: usize,
    change: ClickChange,
    cx: &mut Context<SheprdWindow>,
) {
    match change {
        ClickChange::Shared(change) => apply(view, index, change, cx),
        ClickChange::AnotherSelected => {
            view.selected_endpoint = (0..view.endpoints.len())
                .find(|other| *other != index)
                .unwrap();
            view.reset_selected(cx);
        }
        ClickChange::GenerationOnly => view.endpoints[index].generation += 1,
        ClickChange::InboxOnly => {
            let endpoint = &mut view.endpoints[index];
            endpoint.connection.inbox = Arc::new(Mutex::new(endpoint.live.clone()));
        }
    }
}
