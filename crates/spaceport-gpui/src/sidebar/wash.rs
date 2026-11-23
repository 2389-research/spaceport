//! A host's colour over its rows, under the path the app's painters have
//! always named it by. Which wash and dimming a host gets lives in
//! `spaceport-sidebar`; the app decides which host holds the selected
//! workspace and paints the wash under every row it owns.

pub(super) use spaceport_sidebar::wash::HostMark;

/// The resting wash's alpha, for the paint probes that read it.
#[cfg(test)]
pub(super) use spaceport_sidebar::wash::REST_ALPHA;
