//! Density and style policy, under the path the app's painters have always
//! named it by. The numbers, the highlight chrome, and the badge box they draw
//! with live in `spaceport-sidebar`; the app decides which of them a window
//! uses and paints every row with them.

pub(crate) use spaceport_sidebar::layout::{Highlight, SidebarLook, SidebarMetrics, for_config};

#[cfg(any(test, feature = "integration-test"))]
pub(crate) use spaceport_sidebar::layout::for_mode;
