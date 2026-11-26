#![allow(clippy::unwrap_used, clippy::expect_used)]
mod copper;
mod plasma;
mod ramp;
mod rotozoom;
mod scroller;
mod starfield;

/// Three stops every layer's test ramps through: dark, blue, light.
pub(super) const STOPS: [u32; 3] = [0x101020, 0x4050a0, 0xf0e0d0];
