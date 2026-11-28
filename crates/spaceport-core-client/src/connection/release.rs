// ABOUTME: Closes the channels of dropped streams from the connection worker.
// ABOUTME: Keeps each CLOSE behind accepted data and off reused channels.
//! A dropped stream only records its release; the worker writes the CLOSE.
//! A destructor never waits for outbound capacity or writes to the socket.
//! The worker writes the CLOSE once every frame the queue held when
//! it saw the release has gone out, so it never overtakes accepted data.

use super::{Shared, Worker, locked};
use crate::{
    Result,
    frame::{self, Kind},
    stream::Route,
};
use std::{
    io::Write,
    sync::{Arc, Weak, atomic::Ordering},
};

/// A released route waiting for the frames queued ahead of it.
pub(super) struct Closing {
    channel: u32,
    /// The worker's written-frame count after which the CLOSE may go.
    pub(super) due: u64,
    route: Weak<Route>,
}

impl Shared {
    /// Each registered route belongs to one stream, which releases it once.
    pub(crate) fn release(&self, channel: u32, route: &Arc<Route>) {
        let mut released = locked(&self.released);
        // The daemon can close a route and reuse its channel before the
        // application drops the old stream. Only the current owner owes a CLOSE.
        if !self.lost.load(Ordering::Acquire)
            && locked(&self.routes)
                .get(&channel)
                .is_some_and(|current| Arc::ptr_eq(current, route))
        {
            released.push((channel, Arc::downgrade(route)));
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

    /// Writes one due CLOSE if the daemon still has the same route. A weak
    /// reference identifies it without retaining an already closed inbox.
    pub(super) fn close_released(&mut self, closing: Closing) -> Result<bool> {
        {
            let mut routes = locked(&self.shared.routes);
            if !routes
                .get(&closing.channel)
                .is_some_and(|route| std::ptr::eq(Arc::as_ptr(route), closing.route.as_ptr()))
            {
                return Ok(false);
            }
            routes.remove(&closing.channel);
        }
        self.stream
            .write_all(&frame::encode(closing.channel, Kind::Close, &[])?)?;
        Ok(true)
    }
}
