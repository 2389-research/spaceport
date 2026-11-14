// ABOUTME: Tests the app-only conversation owner with isolated state and peers.
// ABOUTME: Keeps provider and personal configuration outside fixture tests.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use sheprd_chat_protocol::EndpointId;
mod durable;
mod policy;
mod recovery;

fn checkpoint() -> Checkpoint {
    Checkpoint {
        version: sheprd_chat_protocol::ask::VERSION,
        conversation_id: uuid::Uuid::new_v4(),
        helper_run_id: uuid::Uuid::new_v4(),
        endpoint: EndpointId::random().to_string(),
        pi_session: None,
        pi_session_file: None,
        request: None,
        transcript: sheprd_chat_protocol::Transcript::new("Pi"),
        outcome: Outcome::Ready,
    }
}
