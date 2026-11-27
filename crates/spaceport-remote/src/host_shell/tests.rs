// ABOUTME: Wires the host shell's tests: step frames, search paths, local children and WSL.
// ABOUTME: Unix-only cases run the shell as an isolated local child.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;

#[cfg(unix)]
mod frames;
#[cfg(unix)]
mod linuxbrew_path;
#[cfg(unix)]
mod remote_hosts;
mod wsl_hosts;
