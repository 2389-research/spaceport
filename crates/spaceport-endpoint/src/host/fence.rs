//! The connection a deferred action started on: the values that name it, and
//! whether one still names another. The predicates that ask a *window* whether
//! a fence holds stay there, because only a window has an endpoint list.
//!
//! Three kinds of action ask three different questions:
//!
//! - [`InputFence`]: input for the selected connection (a file transfer, a
//!   WSL drop, an image paste). It holds while the same endpoint is selected
//!   on the same connection and daemon boot, with no selection in between.
//! - [`SelectionFence`]: a held terminal mouse gesture. It compares the
//!   window's own record of the selected connection, not the host's, so it
//!   differs from an `InputFence` while the two disagree.
//! - [`ConnectionRef`]: a notice or card. It names the connection it came
//!   from, selected or not, and reaches it for as long as it exists.

use spaceport_live::LiveState;
use std::sync::{Arc, Mutex, Weak};

/// The selected connection that input was meant for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputFence {
    /// The daemon boot the input is addressed to.
    pub boot: String,
    /// The host the input is addressed to.
    pub endpoint: String,
    /// Which selection this began in.
    pub epoch: u64,
    /// The host's connection generation it began on.
    pub generation: u64,
}

/// The selection a held mouse gesture began in, as the window recorded it.
/// It has no host term and reads the window's `selected_generation`, which
/// trails the host's generation until polling resets the window onto a
/// replaced connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionFence {
    /// The daemon boot the gesture's input is addressed to.
    pub boot: String,
    /// Which selection this began in.
    pub epoch: u64,
    /// The window's record of the connection generation it began on.
    pub generation: u64,
}

/// One host's connection, whether or not it is selected.
#[derive(Clone)]
pub struct ConnectionRef {
    endpoint: String,
    generation: u64,
    inbox: Arc<Mutex<LiveState>>,
}

/// A [`ConnectionRef`] that does not keep the connection's state alive.
#[derive(Clone)]
pub struct WeakConnectionRef {
    endpoint: String,
    generation: u64,
    inbox: Weak<Mutex<LiveState>>,
}

impl ConnectionRef {
    pub fn new(endpoint: String, generation: u64, inbox: Arc<Mutex<LiveState>>) -> Self {
        Self {
            endpoint,
            generation,
            inbox,
        }
    }

    pub fn downgrade(&self) -> WeakConnectionRef {
        WeakConnectionRef {
            endpoint: self.endpoint.clone(),
            generation: self.generation,
            inbox: Arc::downgrade(&self.inbox),
        }
    }

    /// Whether this names the connection `endpoint`, `generation` and `inbox`
    /// describe: the same host, the same attempt, the same mailbox.
    pub fn names(&self, endpoint: &str, generation: u64, inbox: &Arc<Mutex<LiveState>>) -> bool {
        self.endpoint == endpoint
            && self.generation == generation
            && Arc::ptr_eq(&self.inbox, inbox)
    }

    /// A reference with terms a test chose, such as a retired generation.
    #[cfg(any(test, feature = "test-support"))]
    pub fn for_test(endpoint: &str, generation: u64, inbox: &Arc<Mutex<LiveState>>) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
            generation,
            inbox: inbox.clone(),
        }
    }
}

impl WeakConnectionRef {
    /// The reference, while the connection's state is still alive.
    pub fn upgrade(&self) -> Option<ConnectionRef> {
        Some(ConnectionRef {
            endpoint: self.endpoint.clone(),
            generation: self.generation,
            inbox: self.inbox.upgrade()?,
        })
    }
}
