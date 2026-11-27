// ABOUTME: Exposes a native Herdr session to Swift through UniFFI.
// ABOUTME: Keeps protocol decoding and action targeting inside Rust.
mod controller;
pub(crate) use controller::history;
mod idle;
mod projection;
mod roster;
pub use roster::MobileRosterRequest;
mod ssh;

pub use controller::{
    MobileError, MobileSession,
    pane_tools::{MobilePaneAction, MobilePaneActions},
};
pub use history::{HistoryCursor, HistoryMatch, HistorySearch};
pub use idle::{IdleAgeGroup, idle_age_group};
pub use projection::{
    HostChallenge, InputTarget, MobileCell, MobilePane, MobilePaneMap, MobilePaneRect,
    MobileScroll, MobileTab, MobileTerminal, MobileView, MobileWorkspace,
};
pub use ssh::*;
uniffi::setup_scaffolding!();

#[cfg(test)]
use {
    controller::geometry,
    projection::{OwnedNavigationTarget, Projection},
};
#[cfg(test)]
mod tests;
