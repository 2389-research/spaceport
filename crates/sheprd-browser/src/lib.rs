//! Native browser tabs: what each tab shows, where it lives, and how a
//! window's workspaces are split into groups of them.
//!
//! A tab's address, kind and title are the app's own saved state, which this
//! crate reads and writes; a window's split layouts, tab strips and their
//! motions are per window. Terminal groups, the connection and the daemon
//! stay the app's: it hands a call what it may read and the crate changes
//! only what a tab owns.
mod closed;
mod error;
mod group_motion;
mod groups;
mod hidden;
mod layouts;
mod location;
mod store;
mod tab_appear;
mod tab_body;
mod tab_scroll;
mod web_url;

pub use closed::{ClosedHerdrTab, ClosedTab};
pub use error::{Error, Result};
pub use group_motion::{Fold, GroupMotion};
pub use groups::{GroupId, GroupIds, Layout, Pick, SavedLayout, Shown, Slot};
pub use hidden::Hidden;
pub use layouts::Layouts;
pub use location::Location;
pub use store::{CoreGroupKey, Scope, Store, Tab, TabId};
pub use tab_appear::{Leaving, Listed, TabAppear};
pub use tab_body::{Cells, Frame, List, Span, TabBodies};
pub use tab_scroll::{TabScroll, Thumb, ThumbDrag};
pub use web_url::WebUrl;
