//! Recovering connections after the machine sleeps or its network changes. A
//! link that died meanwhile is otherwise found only by the routine health
//! check of a quiet link, and a dropped one waits out a backoff earned while
//! no network could have answered.
//!
//! Whether one is due comes from `spaceport_endpoint`'s clock, and whether
//! the network changed from its monitor; this module only acts on both.

use crate::SpaceportWindow;
use std::time::{Instant, SystemTime};

pub(super) use spaceport_endpoint::{NetworkChanges, Trigger, network_changes};

impl SpaceportWindow {
    /// After a sleep or a network change, have every endpoint recover rather
    /// than wait out its backoff: the network the user woke to, or just
    /// joined, may be the one it was waiting for. Each endpoint recovers in
    /// its own poll, once it has taken what its worker reported, so a
    /// recovery never acts on a status one update old.
    pub(crate) fn recover_connections(
        &mut self,
        instant: Instant,
        wall: SystemTime,
        network: NetworkChanges,
    ) -> bool {
        let Some(trigger) = self.wake.due(instant, wall, network) else {
            return false;
        };
        match trigger {
            Trigger::Sleep => {
                tracing::info!(category = "wake", "machine woke; checking connections");
            }
            Trigger::NetworkRestored => {
                tracing::info!(
                    category = "network",
                    "network came back; checking connections"
                );
            }
            Trigger::PathChanged => {
                tracing::info!(
                    category = "network",
                    "network path changed; checking live connections"
                );
            }
        }
        for endpoint in &mut self.endpoints {
            endpoint.recovery = endpoint.recovery.max(Some(trigger));
        }
        true
    }
}

#[cfg(test)]
mod tests;
