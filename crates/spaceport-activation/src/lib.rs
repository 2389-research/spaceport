//! The activation fence, and the route a staged focus request walks.
//!
//! Both clients drive one Herdr daemon the same way: ask it to focus a
//! destination, ask it to activate a terminal surface with an ordered barrier,
//! and keep input off until the barrier's acknowledgement and the projection it
//! names belong to each other. The desktop and the phone each wrote that rule —
//! the phone's `Projection` and its `go_to`/`resume` modules, the desktop's
//! `spaceport-live::activation` — and the copies had already drifted: one
//! retried a barrier the queue refused and the other dropped the connection,
//! and one collapsed four failure causes into a single error.
//!
//! So the rule lives here, below both of them, with no GPUI and no transport:
//!
//! - [`Fence`] is the lifecycle of one activation, and [`Transition`] is the
//!   only writer for it. [`Failure`] names why it failed, and [`TIMEOUT`] is
//!   the budget one barrier gets.
//! - [`Route`] is a destination reached one level per request, for a client
//!   that has to confirm each hop. [`ROUTE_BUDGET`] is the budget the whole
//!   walk gets.
//! - [`NavigationTarget`] is where a focus request is aimed, and
//!   [`SurfaceActivation`] is the activation one barrier was asked for.
//!
//! What is not here: the projection a fence is fenced against (both clients
//! keep their own), the events that drive the transitions (each client's
//! reducer reads its own), and every word — a client says what a [`Failure`]
//! means in its own language to its own user.

mod fence;
mod route;

pub use fence::{Failure, Fence, SurfaceActivation, TIMEOUT, Transition};
pub use route::{ROUTE_BUDGET, Route, Step};
pub use spaceport_client::navigation::{NavigationTarget, OwnedNavigationTarget};

#[cfg(test)]
mod tests;
