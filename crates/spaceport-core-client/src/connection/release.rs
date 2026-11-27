// ABOUTME: Closes the channels of dropped streams from the connection worker.
// ABOUTME: Keeps each CLOSE behind accepted data and off reused channels.
//! A dropped stream only records its release; the worker writes the CLOSE.
//! A destructor therefore never blocks, and a full outbound queue cannot
//! lose the close. The worker writes it once every frame the queue held when
//! it saw the release has gone out, so it never overtakes accepted data.

use super::{Shared, Worker, locked};
use crate::{
    Result,
    frame::{self, Kind},
    stream::Route,
};
use std::{
    io::Write,
    sync::{Arc, atomic::Ordering},
};

/// A released route waiting for the frames queued ahead of it.
pub(super) struct Closing {
    channel: u32,
    /// The worker's written-frame count after which the CLOSE may go.
    due: u64,
    route: Arc<Route>,
}

impl Shared {
    /// Records that the stream owning `route` on `channel` is gone. Each
    /// registered route belongs to one stream, which releases it once, so
    /// pending releases stay bounded by the registered routes.
    pub(crate) fn release(&self, channel: u32, route: Arc<Route>) {
        let mut released = locked(&self.released);
        // A lost connection already ended every route; nothing is left to close.
        if !self.lost.load(Ordering::Acquire) {
            released.push((channel, route));
        }
    }
}

impl<S: Write> Worker<S> {
    /// Moves new releases into `closing`, each due once the frames queued
    /// now are written. The stream queued its last frame before releasing,
    /// so the queue's length read after taking the release counts it.
    pub(super) fn take_releases(&mut self) {
        let released = std::mem::take(&mut *locked(&self.shared.released));
        if released.is_empty() {
            return;
        }
        let due = self.written + self.queue.len() as u64;
        self.closing
            .extend(released.into_iter().map(|(channel, route)| Closing {
                channel,
                due,
                route,
            }));
    }

    /// Writes the CLOSE of every release whose frames have gone out, unless
    /// the core already closed that channel. Returns whether it wrote any.
    pub(super) fn close_released(&mut self) -> Result<bool> {
        let written = self.written;
        let (ready, waiting) = std::mem::take(&mut self.closing)
            .into_iter()
            .partition::<Vec<_>, _>(|closing| closing.due <= written);
        self.closing = waiting;
        let mut wrote = false;
        for closing in ready {
            // A route the core closed is gone, or another stream's now that
            // the channel was reused; either way this CLOSE is not owed.
            let owed = {
                let mut routes = locked(&self.shared.routes);
                let ours = routes
                    .get(&closing.channel)
                    .is_some_and(|route| Arc::ptr_eq(route, &closing.route));
                if ours {
                    routes.remove(&closing.channel);
                }
                ours
            };
            if owed {
                self.stream
                    .write_all(&frame::encode(closing.channel, Kind::Close, &[])?)?;
                wrote = true;
            }
        }
        Ok(wrote)
    }
}
