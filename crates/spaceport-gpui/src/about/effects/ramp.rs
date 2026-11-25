//! Colours taken from a short set of stops, so a layer's palette is a value
//! its maths returns rather than a table it indexes. Three stops hold what
//! every layer needs: a dark end, a bright one, and a hue between them.

/// `k` of the way from `from` to `to`, with `k` clamped to 0..1.
pub(in crate::about) fn mix(from: u32, to: u32, k: f32) -> u32 {
    let k = k.clamp(0., 1.);
    let channel = |shift: u32| {
        let from = ((from >> shift) & 0xff) as f32;
        let to = ((to >> shift) & 0xff) as f32;
        (from + (to - from) * k).round() as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

/// The colour of `stops` at `v`: the first stop at 0, the last at 1, and the
/// mix of the two stops either side in between.
pub(in crate::about) fn ramp(stops: &[u32; 3], v: f32) -> u32 {
    let at = v.clamp(0., 1.) * (stops.len() - 1) as f32;
    let below = (at.floor() as usize).min(stops.len() - 2);
    mix(stops[below], stops[below + 1], at - below as f32)
}
