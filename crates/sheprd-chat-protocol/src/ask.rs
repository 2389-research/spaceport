// ABOUTME: Defines the durable app-owned Ask Spaceport conversation checkpoint.
// ABOUTME: Distinguishes agent receipts from queueing and uncertain outcomes.
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CommandId, Transcript};

pub const VERSION: u32 = 1;
pub const TOOLS: &[&str] = &[
    "spaceport_list_sessions",
    "spaceport_read_conversation",
    "spaceport_read_terminal",
];
pub const MAX_CHECKPOINT_BYTES: usize = 4 * 1024 * 1024;
pub const CHECKPOINT_FILE: &str = "checkpoint.json";
pub const QUIT_FILE: &str = "quit";
pub const STOP_FILE: &str = "stop";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Checkpoint {
    pub version: u32,
    pub conversation_id: Uuid,
    pub helper_run_id: Uuid,
    pub endpoint: String,
    pub pi_session: Option<String>,
    pub pi_session_file: Option<PathBuf>,
    pub request: Option<Request>,
    pub transcript: Transcript,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub id: CommandId,
    pub text: String,
    pub phase: Phase,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Intent,
    Accepted,
    Interrupted,
    Completed,
    Stopped,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Ready,
    Answering,
    Completed,
    Stopped,
    Interrupted,
    Unknown,
    Failed,
}
