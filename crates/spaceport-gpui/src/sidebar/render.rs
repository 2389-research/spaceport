// ABOUTME: Lays out prepared sidebar rows, headings and resize controls.
// ABOUTME: Keeps runtime-specific sections behind the running session's feature choices.
//! Laying out the sidebar: the two lists, their headings, and the drag handle
//! that resizes the panel. Render works from prepared state and the bounded
//! caches only, in four stages: `sidebar_measure` reads the frame's look and
//! choices, `sidebar_rows` builds the rows, `sidebar_settle` pins and reveals
//! them against the last frame's geometry, and `sidebar_frame` paints them.

use super::agents::Indicators;
use crate::SpaceportWindow;
use gpui::{prelude::*, *};

mod agent_rows;
mod device_tree;
mod edges;
mod frame;
pub(super) mod header;
mod measure;
mod nested_agents;
mod rows;
mod settle;

pub(crate) use nested_agents::NestedReveal;

impl SpaceportWindow {
    pub(crate) fn render_sidebar(
        &self,
        indicators: Indicators,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let m = self.sidebar_measure(window, cx);
        let rows = self.sidebar_rows(&m, indicators, cx);
        let settled = self.sidebar_settle(&m, &rows, window, cx);
        self.sidebar_frame(&m, rows, settled, window, cx)
    }
}
