// ABOUTME: Names the connection a deferred action started on, and checks it still holds.
// ABOUTME: One predicate per kind of fence, so each action adds only its own readiness terms.
//! An action that starts on one connection and finishes later must not act
//! on another. The values that name a connection are the crate's
//! (`spaceport_endpoint`'s `fence`), and so is the comparison that says whether
//! one [`ConnectionRef`] still names another; the predicates below ask a
//! *window*, because only a window has an endpoint list and a selection.
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
use crate::SpaceportWindow;
pub(crate) use spaceport_endpoint::{ConnectionRef, InputFence, SelectionFence, WeakConnectionRef};

impl Endpoint {
    /// This endpoint's current connection.
    pub(crate) fn connection_ref(&self) -> ConnectionRef {
        ConnectionRef::new(
            self.id.clone(),
            self.generation,
            self.connection.inbox.clone(),
        )
    }
}

impl SpaceportWindow {
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
            source.names(
                &endpoint.id,
                endpoint.generation,
                &endpoint.connection.inbox,
            )
        })
    }
}

#[cfg(test)]
pub(crate) mod changes;
