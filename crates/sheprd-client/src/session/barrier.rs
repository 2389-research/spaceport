//! A surface activation written behind an unacknowledged synchronous focus
//! request. Activation advances the revision that fences deferred navigation,
//! so it must never follow any other request early. Every transition of the
//! request lease is made here: a request sent, a response routed, a resend.

use super::{Pending, Session};
use crate::{
    Error, Result,
    event::ClientEvent,
    handle::Command,
    limits::{MAX_RESPONSE_BYTES, SLOW_REQUEST},
    method::Method,
};
use serde_json::Value;
use std::time::Instant;

pub(crate) struct Barrier {
    pub(crate) pending: Pending,
    pub(super) request: Vec<u8>,
    reply: Reply,
}

/// What the daemon has said about the activation so far.
enum Reply {
    Awaiting,
    /// Delivered only after the focus response, preserving send order. Its
    /// raw bytes remain in `pending` and count toward the response bound.
    Held(Value),
    /// A daemon without the busy-check exemption refused it unprocessed.
    Refused,
}

impl Barrier {
    pub(crate) fn new(pending: Pending, request: Vec<u8>) -> Self {
        Self {
            pending,
            request,
            reply: Reply::Awaiting,
        }
    }

    fn awaits(&self, request_id: &str) -> bool {
        self.pending.id == request_id && matches!(self.reply, Reply::Awaiting)
    }
}

/// Adds one response chunk to the request it answers and returns the responses
/// that are now due, in delivery order: focus always precedes its activation.
/// Both assemblies, and a held activation's bytes, share one bound.
pub(super) fn receive(
    pending: &mut Option<Pending>,
    barrier: &mut Option<Barrier>,
    pipeline_barriers: &mut bool,
    request_id: String,
    final_chunk: bool,
    data: Vec<u8>,
) -> Result<[Option<ClientEvent>; 2]> {
    let total = pending
        .iter()
        .chain(barrier.iter().map(|b| &b.pending))
        .map(|p| p.bytes.len())
        .sum::<usize>();
    if data.len() > MAX_RESPONSE_BYTES.saturating_sub(total) {
        return Err(Error::ResponseLimit);
    }
    if let Some(b) = barrier.as_mut().filter(|b| b.awaits(&request_id)) {
        b.pending.bytes.extend(data);
        if !final_chunk {
            return Ok([None, None]);
        }
        let bytes = std::mem::take(&mut b.pending.bytes);
        let response = completed(&bytes, &request_id, b.pending.started)?;
        if endpoint_busy(&response) {
            tracing::debug!(category = "api", "surface activation resent after focus");
            *pipeline_barriers = false;
            b.reply = Reply::Refused;
            return Ok([None, None]);
        }
        if pending.is_some() {
            // Its bytes stay counted until delivery, so the two responses
            // share one assembly bound.
            b.pending.bytes = bytes;
            b.reply = Reply::Held(response);
            return Ok([None, None]);
        }
        *barrier = None;
        return Ok([delivery(request_id, response), None]);
    }
    let Some(mut request) = pending.take_if(|p| p.id == request_id) else {
        return Err(Error::UnsolicitedResponse);
    };
    request.bytes.extend(data);
    if !final_chunk {
        *pending = Some(request);
        return Ok([None, None]);
    }
    let response = completed(&request.bytes, &request_id, request.started)?;
    let first = delivery(request_id, response);
    Ok(match barrier.take() {
        Some(Barrier {
            pending: activation,
            reply: Reply::Held(held),
            ..
        }) => [first, delivery(activation.id, held)],
        waiting => {
            *barrier = waiting;
            [first, None]
        }
    })
}

fn delivery(request_id: String, response: Value) -> Option<ClientEvent> {
    Some(ClientEvent::Response {
        request_id,
        response,
    })
}

/// Parses a fully assembled response and checks it answers `request_id`.
fn completed(bytes: &[u8], request_id: &str, started: Instant) -> Result<Value> {
    let response: Value = serde_json::from_slice(bytes)?;
    if response.get("id").and_then(Value::as_str) != Some(request_id) {
        return Err(Error::ResponseId);
    }
    let elapsed = started.elapsed();
    let elapsed_ms = elapsed.as_millis() as u64;
    let api_error = response.get("error").is_some_and(|error| !error.is_null());
    if api_error {
        tracing::warn!(elapsed_ms, "API request failed");
    } else if elapsed > SLOW_REQUEST {
        tracing::warn!(elapsed_ms, api_error, "slow API request completed");
    } else {
        tracing::trace!(elapsed_ms, api_error, "API request completed");
    }
    Ok(response)
}

/// The exact stable releases whose sources were checked; any other string,
/// including previews, later versions, and other spellings, stays serialized.
pub(super) fn verified_ordering(server_version: &str) -> bool {
    matches!(
        server_version,
        "0.9.0" | "0.9.1" | "0.9.2" | "0.9.3" | "0.9.4" | "0.9.5"
    )
}

pub(super) fn endpoint_busy(response: &Value) -> bool {
    response
        .pointer("/error/code")
        .is_some_and(|code| code == "endpoint_busy")
}

impl Session {
    pub(crate) fn pipelines(&self, command: &Command) -> bool {
        self.pipeline_barriers
            && self.barrier.is_none()
            && command
                .request
                .as_ref()
                .is_some_and(|request| request.method == Method::ClientShellSurfaceSet)
            && self.pending.as_ref().is_some_and(|pending| {
                matches!(
                    pending.method,
                    Method::PaneFocus | Method::TabFocus | Method::WorkspaceFocus
                )
            })
    }

    /// A refused activation is resent unchanged once focus completes. Focus is
    /// never resent; a reboot or cancellation ends the connection and this state.
    pub(crate) fn take_refused_barrier(&mut self) -> Option<Barrier> {
        if !self.resend_ready() {
            return None;
        }
        self.barrier.take()
    }

    /// Focus has completed and its refused activation waits to be resent.
    pub(crate) fn resend_ready(&self) -> bool {
        self.pending.is_none()
            && self
                .barrier
                .as_ref()
                .is_some_and(|b| matches!(b.reply, Reply::Refused))
    }

    /// A request went on the wire: it takes the lease, or waits as the
    /// activation behind the focus request that holds it.
    pub(super) fn request_sent(&mut self, pending: Pending, request: Vec<u8>) {
        if self.pending.is_some() {
            self.barrier = Some(Barrier::new(pending, request));
        } else {
            self.pending = Some(pending);
        }
    }

    /// The refused activation went on the wire again, as an ordinary request
    /// with a fresh deadline: a second refusal is delivered, not resent.
    pub(super) fn resent(&mut self, barrier: Barrier) {
        self.pending = Some(Pending {
            started: Instant::now(),
            ..barrier.pending
        });
    }
}
