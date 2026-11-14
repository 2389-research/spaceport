// ABOUTME: Names the connection a deferred action started on, and checks it still holds.
// ABOUTME: One predicate per kind of fence, so each action adds only its own readiness terms.
//! An action that starts on one connection and finishes later must not act
//! on another. Three kinds of action ask three different questions, so there
//! are three values, each with one predicate:
//!
//! - [`InputFence`]: input for the selected connection (a file transfer, a
//!   WSL drop, an image paste). It holds while the same endpoint is selected
//!   on the same connection and daemon boot, with no selection in between.
//! - [`SelectionFence`]: a held terminal mouse gesture. It compares the
//!   window's own record of the selected connection, not the endpoint's, so
//!   it differs from an `InputFence` while the two disagree.
//! - [`ConnectionRef`]: a notice or card. It names the connection it came
//!   from, selected or not, and reaches it for as long as it exists.

use super::Endpoint;
use crate::{SheprdWindow, state::LiveState};
use std::sync::{Arc, Mutex, Weak};

/// The selected connection that input was meant for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InputFence {
    /// The daemon boot the input is addressed to.
    pub(crate) boot: String,
    endpoint: String,
    epoch: u64,
    generation: u64,
}

/// The selection a held mouse gesture began in, as the window recorded it.
/// It has no endpoint term and reads `selected_generation`, which trails the
/// endpoint's generation until polling resets the window onto a replaced
/// connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectionFence {
    /// The daemon boot the gesture's input is addressed to.
    pub(crate) boot: String,
    epoch: u64,
    generation: u64,
}

/// One endpoint's connection, whether or not it is selected.
#[derive(Clone)]
pub(crate) struct ConnectionRef {
    endpoint: String,
    generation: u64,
    inbox: Arc<Mutex<LiveState>>,
}

/// A [`ConnectionRef`] that does not keep the connection's state alive.
#[derive(Clone)]
pub(crate) struct WeakConnectionRef {
    endpoint: String,
    generation: u64,
    inbox: Weak<Mutex<LiveState>>,
}

impl ConnectionRef {
    pub(crate) fn downgrade(&self) -> WeakConnectionRef {
        WeakConnectionRef {
            endpoint: self.endpoint.clone(),
            generation: self.generation,
            inbox: Arc::downgrade(&self.inbox),
        }
    }

    /// A reference with terms a test chose, such as a retired generation.
    #[cfg(test)]
    pub(crate) fn for_test(endpoint: &str, generation: u64, inbox: &Arc<Mutex<LiveState>>) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
            generation,
            inbox: inbox.clone(),
        }
    }
}

impl WeakConnectionRef {
    /// The reference, while the connection's state is still alive.
    pub(crate) fn upgrade(&self) -> Option<ConnectionRef> {
        Some(ConnectionRef {
            endpoint: self.endpoint.clone(),
            generation: self.generation,
            inbox: self.inbox.upgrade()?,
        })
    }
}

impl Endpoint {
    /// This endpoint's current connection.
    pub(crate) fn connection_ref(&self) -> ConnectionRef {
        ConnectionRef {
            endpoint: self.id.clone(),
            generation: self.generation,
            inbox: self.connection.inbox.clone(),
        }
    }
}

impl SheprdWindow {
    /// The selected connection as input started now would name it; `None`
    /// without a snapshot.
    pub(crate) fn input_fence(&self) -> Option<InputFence> {
        let endpoint = &self.endpoints[self.selected_endpoint];
        Some(InputFence {
            boot: self.live.snapshot.as_ref()?.boot_id.clone(),
            endpoint: endpoint.id.clone(),
            epoch: self.selection_epoch,
            generation: endpoint.generation,
        })
    }

    /// Whether the selected connection is still the one `fence` named: no
    /// selection since, the same endpoint and connection, the same boot.
    pub(crate) fn fence_holds(&self, fence: &InputFence) -> bool {
        let endpoint = &self.endpoints[self.selected_endpoint];
        fence.epoch == self.selection_epoch
            && fence.generation == endpoint.generation
            && fence.endpoint == endpoint.id
            && self
                .live
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.boot_id == fence.boot)
    }

    /// The selection as a mouse gesture begun now would name it; `None`
    /// without a snapshot.
    pub(crate) fn selection_fence(&self) -> Option<SelectionFence> {
        Some(SelectionFence {
            boot: self.live.snapshot.as_ref()?.boot_id.clone(),
            epoch: self.selection_epoch,
            generation: self.selected_generation,
        })
    }

    /// Whether the window's record of its selection is still what `fence`
    /// named: no selection or reset since, and the same boot.
    pub(crate) fn selection_fence_holds(&self, fence: &SelectionFence) -> bool {
        fence.epoch == self.selection_epoch
            && fence.generation == self.selected_generation
            && self
                .live
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.boot_id == fence.boot)
    }

    /// The endpoint whose current connection is `source`, selected or not.
    pub(crate) fn endpoint_for(&self, source: &ConnectionRef) -> Option<usize> {
        self.endpoints.iter().position(|endpoint| {
            endpoint.id == source.endpoint
                && endpoint.generation == source.generation
                && Arc::ptr_eq(&endpoint.connection.inbox, &source.inbox)
        })
    }
}

#[cfg(test)]
pub(crate) mod changes;
