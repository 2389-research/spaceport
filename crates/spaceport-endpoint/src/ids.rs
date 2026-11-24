//! The IDs saved hosts are keyed by. A saved SSH host is its catalog
//! profile's, so per-device credentials and the `herdr machine` commands name
//! the same thing; Local and the saved WSL distributions have IDs of their
//! own, and none may collide with a catalog's.

/// The endpoint Local is, which every device list keeps as endpoint 0.
pub const LOCAL: &str = "local";
/// Saved SSH endpoints are keyed `ssh:<profile-id>`, so no catalog ID can
/// collide with [`LOCAL`].
pub const SAVED_PREFIX: &str = "ssh:";
/// Saved WSL distributions are keyed `wsl:<distribution>`; a distribution is
/// saved at most once.
pub const WSL_PREFIX: &str = "wsl:";

/// The catalog profile ID behind a saved SSH endpoint's ID, which is what the
/// `herdr machine` commands and per-device credentials are keyed by.
pub fn saved_profile_id(endpoint_id: &str) -> Option<&str> {
    endpoint_id
        .strip_prefix(SAVED_PREFIX)
        .filter(|id| spaceport_client::valid_profile_id(id))
}
