//! Dragging a workspace row to reorder it: the press that arms it, the lift,
//! the pointer that moves it, and the release that sends its move. The drag
//! itself, the plan it reorders and the gap a drop lands in are
//! `spaceport-sidebar`'s; this reads the window's connection and its snapshot
//! and sends the request over the selected endpoint.

use crate::SpaceportWindow;
use crate::i18n::fl;
use gpui::{Context, Pixels, Point};
use spaceport_reorder::{LIFT_DELAY, LIFT_DISTANCE, SETTLE_TIMEOUT};
use spaceport_sidebar::reorder::{Phase, Plan, Target, WorkspaceDrag};

impl SpaceportWindow {
    /// Arms a reorder for a left press on a row of the selected endpoint. One
    /// reorder runs at a time: a device header already pressed keeps the
    /// gesture, as a pressed row keeps it from a header.
    pub(super) fn press_workspace(
        &mut self,
        workspace: &str,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(snapshot) = &self.live.snapshot else {
            return;
        };
        if self.menu.page.is_some()
            || self.device_drag.is_some()
            || !self.live.status.is_connected()
            || Plan::new(&snapshot.workspaces, workspace).is_none()
        {
            return;
        }
        let lift = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(LIFT_DELAY).await;
            let _ = this.update(cx, |this, cx| this.lift_workspace(cx));
        });
        let drag = WorkspaceDrag::press(snapshot.boot_id.clone(), workspace, position, lift);
        self.workspace_drag = Some(drag);
    }

    fn lift_workspace(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = &mut self.workspace_drag
            && matches!(drag.phase, Phase::Pressed)
        {
            drag.phase = Phase::Floating { target: None };
            // The resting pointer is carrying a row, not asking for its menu.
            self.hover = None;
            cx.notify();
        }
    }

    /// Follows the pointer. `target` resolves a lifted drag's gap from how far
    /// the card has moved, against the geometry the sidebar last laid out.
    /// Returns whether the move belongs to the drag, so the terminal and hover
    /// never see it.
    pub(super) fn move_workspace_drag(
        &mut self,
        position: Point<Pixels>,
        held: bool,
        target: impl FnOnce(Pixels) -> Option<Target>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(drag) = self.workspace_drag.as_mut().filter(|d| !d.settling()) else {
            return false;
        };
        // A release outside the window never reaches the sidebar.
        if !held {
            let lifted = drag.floating();
            self.workspace_drag = None;
            cx.notify();
            return lifted;
        }
        drag.pointer = position;
        let moved = position - drag.origin;
        if f32::from(moved.x.abs().max(moved.y.abs())) > LIFT_DISTANCE {
            self.lift_workspace(cx);
        }
        let Some(drag) = self.workspace_drag.as_mut() else {
            return false;
        };
        let offset = drag.offset();
        let Phase::Floating { target: gap } = &mut drag.phase else {
            return false;
        };
        *gap = target(offset);
        cx.notify();
        true
    }

    /// Ends the drag on release, sending its move if it was lifted over a gap.
    /// Returns whether it had lifted, in which case the release is not a click.
    /// A row already dropped stays as it is: its preview holds until the
    /// daemon answers, and this release belongs to some later click.
    pub(super) fn release_workspace_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.workspace_drag.take_if(|drag| !drag.settling()) else {
            return false;
        };
        let Phase::Floating { target } = &drag.phase else {
            return false;
        };
        cx.notify();
        let Some(target) = target.clone() else {
            return true;
        };
        let connection = &self.endpoints[self.selected_endpoint].connection;
        let (Some(handle), Some(snapshot)) = (&connection.handle, &self.live.snapshot) else {
            return true;
        };
        if snapshot.boot_id != drag.boot {
            return true;
        }
        let reorder = target
            .request
            .reorder(&snapshot.workspaces, self.live.supports_workspace_move);
        match handle.reorder_workspaces(&drag.boot, &reorder) {
            Ok(_) => {
                let order = snapshot
                    .workspaces
                    .iter()
                    .map(|workspace| workspace.workspace_id.clone())
                    .collect();
                let settle = cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(SETTLE_TIMEOUT).await;
                    let _ = this.update(cx, |this, cx| {
                        this.workspace_drag = None;
                        cx.notify();
                    });
                });
                self.workspace_drag = Some(drag.dropped(target, order, settle));
            }
            Err(error) => {
                self.notices.show(
                    crate::notice::Cause::Command,
                    fl!("sidebar-workspace-move-not-sent", error = error.to_string()),
                );
            }
        }
        true
    }

    /// Forgets a dropped row once the window's order is no longer the one it
    /// was dropped on: the daemon answered, so nothing is left to preview and
    /// the next gesture need not wait out the timeout. Called on every tick,
    /// after the window takes the selected endpoint's state.
    pub(crate) fn finish_workspace_settle(&mut self) {
        let workspaces = self
            .live
            .snapshot
            .as_ref()
            .map_or(&[][..], |snapshot| &snapshot.workspaces);
        self.workspace_drag
            .take_if(|drag| drag.settling() && !drag.previewing(workspaces));
    }

    /// Drops a lifted row where it came from, and forgets a press that has
    /// not lifted yet, so its timer cannot lift it later. A dropped row's move
    /// is already sent, so its preview stays. Returns whether one was lifted:
    /// only then is the key the drag's.
    pub(crate) fn cancel_workspace_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.workspace_drag.take_if(|drag| !drag.settling()) else {
            return false;
        };
        let lifted = drag.floating();
        if lifted {
            cx.notify();
        }
        lifted
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
