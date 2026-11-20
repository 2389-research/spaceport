//! What the browser may read of its window for one call.
//!
//! [`Browser`] owns everything a browser tab needs and changes it in place. The
//! window owns the endpoints, the shown snapshot, the tab store and the clock,
//! so every call that reads one of those takes [`BrowserHosts`] instead of the
//! window: it is the whole of the seam the browser has to the app.

use super::{CoreGroupKey, Scope, Store};
use sheprd_client::protocol::ClientShellSnapshot;
use std::time::Instant;

/// What one window offers the browser for a single call. The app owns the
/// window and the store; the browser only reads them.
pub struct BrowserHosts<'a> {
    /// Whether this build shows herdr-core features: tabs that need it are
    /// left out of strips when false.
    pub core_enabled: bool,
    /// The herdr-core group this window shows, or none for a legacy workspace.
    pub core_group: Option<&'a CoreGroupKey>,
    /// `sheprd_motion::now()`, read once per call so every animation in it
    /// agrees.
    pub now: Instant,
    /// The scope of the endpoint this window shows, as its socket or catalog
    /// ID names it: what a tab of this window belongs to.
    pub scope: Option<Scope>,
    /// The newest metadata of the shown endpoint, or none before it connects.
    pub snapshot: Option<&'a ClientShellSnapshot>,
    /// The narrowest a tab draws in the layout in use, `TabLayout::min_width`,
    /// which a growing tab starts from.
    pub tab_min_width: f32,
    /// The window's tab store, read for a caller with a window context; none
    /// without one, and none while no store is installed.
    pub tabs: Option<&'a Store>,
    /// The workspace this window shows browser tabs for, or none: no workspace
    /// is focused, a herdr-core group is shown instead, or the caller had no
    /// window context.
    pub workspace: Option<(Scope, String)>,
}
