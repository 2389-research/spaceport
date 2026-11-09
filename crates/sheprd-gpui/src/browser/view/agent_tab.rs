//! Workspace navigation for retained tab commands.

use crate::{NavigationTarget, SheprdWindow};
use gpui::Context;

impl SheprdWindow {
    /// Whether the window shows `workspace` of endpoint `index` now.
    pub(crate) fn shows_workspace(&self, index: usize, workspace: &str) -> bool {
        index == self.selected_endpoint
            && self
                .live
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.focused_workspace_id.as_deref())
                == Some(workspace)
    }

    /// Switches to `workspace` of endpoint `index`, unless the window shows
    /// it already. The switch lands with a later snapshot.
    pub(crate) fn go_to_workspace(
        &mut self,
        index: usize,
        workspace: &str,
        cx: &mut Context<Self>,
    ) {
        self.leave_core_group(cx);
        if self.shows_workspace(index, workspace) {
            return;
        }
        if index != self.selected_endpoint {
            let endpoint = self.endpoints[index].id.clone();
            self.navigate_endpoint(&endpoint, NavigationTarget::Workspace(workspace), cx);
        } else {
            self.navigate(NavigationTarget::Workspace(workspace), cx);
        }
    }
}
