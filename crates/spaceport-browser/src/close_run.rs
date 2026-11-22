//! A run of closes, one tab after another, as Close Others makes: the state
//! the window's driver carries between ticks. The driver, and what a closed
//! tab means to the app, are the window's.
use super::{Pick, Scope};
use std::{collections::VecDeque, time::Instant};

pub struct CloseRun {
    pub workspace: (Scope, String),
    pub rest: VecDeque<Pick>,
    /// The Herdr tab whose close is being decided, or answered by the daemon.
    pub current: Option<String>,
    /// When its close went to the daemon.
    pub sent: Option<Instant>,
}
