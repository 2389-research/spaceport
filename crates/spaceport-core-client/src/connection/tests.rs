// ABOUTME: Regression tests for Core call ownership and stream cleanup.
// ABOUTME: Uses bounded coordination and explicitly isolated protocol fixtures.

use super::*;
use crate::tests::connected;
use std::thread;

mod abandoned;
mod lifecycle;
mod live;
mod release;
