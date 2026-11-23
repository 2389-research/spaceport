//! Dragging a device's header to reorder the list: the press that arms it, the
//! list folding to its headers, and the release that moves the device. The
//! drag, its slides and the gap a drop lands in are `spaceport-sidebar`'s;
//! this lists the headers the window shows and moves the device it drops.

use crate::SpaceportWindow;
use gpui::{prelude::*, *};
use spaceport_reorder::{Beside, LIFT_DELAY, LIFT_DISTANCE, preview};
use spaceport_sidebar::device_drag::{DeviceDrag, DeviceDrop, DevicePlan, gap, window_top};
use std::collections::HashMap;

impl SpaceportWindow {
    /// Arms a reorder for a left press on device `index`'s header. `row` is
    /// the header's place in the spaces list.
    pub(super) fn press_device(
        &mut self,
        index: usize,
        row: usize,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        // A device listed alone, by the filter or a search, has nowhere to
        // go, so its header stays a plain click.
        let devices = !self.config.layout.mode.lists_workspaces();
        let query = if devices && self.config.show_sidebar_search {
            self.device_tree_query(cx)
        } else {
            String::new()
        };
        if self.listed_headers(devices, &query).len() < 2
            || self.menu.page.is_some()
            || self.workspace_drag.is_some()
        {
            return;
        }
        let Some(endpoint) = self.endpoints.get(index) else {
            return;
        };
        let endpoint = endpoint.id.clone();
        let top = window_top(&self.sidebar_scroll[0], row).unwrap_or(position.y);
        let lift = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(LIFT_DELAY).await;
            let _ = this.update(cx, |this, cx| this.lift_device(cx));
        });
        let drag = DeviceDrag::press(endpoint, position, position.y - top, lift);
        self.device_drag = Some(drag);
    }

    fn lift_device(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = &mut self.device_drag
            && !drag.lifted
        {
            drag.lifted = true;
            self.hover = None;
            cx.notify();
        }
    }

    /// Each header the spaces list shows, by endpoint index, with its row in
    /// the folded list, where every header is one row. `devices` and `query`
    /// list the headers as render does.
    fn listed_headers(&self, devices: bool, query: &str) -> Vec<(usize, usize)> {
        self.listed_hosts()
            .filter(|&(index, _, _)| !devices || self.device_listed(index, query))
            .enumerate()
            .map(|(row, (index, _, _))| (index, row))
            .collect()
    }

    /// The folded list's headers for a lifted device, or `None` when no
    /// device is lifted or fewer than two can move. `devices` and `query`
    /// list the headers as render does.
    pub(super) fn device_plan(&self, devices: bool, query: &str) -> Option<DevicePlan> {
        let drag = self.device_drag.as_ref().filter(|drag| drag.lifted)?;
        let units = self.listed_headers(devices, query);
        let dragged = units
            .iter()
            .position(|&(index, _)| self.endpoints[index].id == drag.endpoint)?;
        if units.len() < 2 {
            return None;
        }
        let scroll = &self.sidebar_scroll[0];
        let measured = drag.mark_measured();
        let heights: Vec<f32> = units
            .iter()
            .map(|&(_, row)| {
                scroll
                    .bounds_for_item(row)
                    .filter(|_| measured)
                    .map_or(0., |bounds| f32::from(bounds.size.height))
            })
            .collect();
        let slot = drag.target.as_ref().map_or(dragged, |(slot, _)| *slot);
        let previewed = preview(&heights, dragged, slot);
        let now = crate::motion::now();
        let mut plan = DevicePlan {
            shifts: HashMap::new(),
            drop: DeviceDrop {
                rows: Vec::new(),
                ids: Vec::new(),
                dragged,
            },
            sliding: !measured,
        };
        for (unit, &(index, row)) in units.iter().enumerate() {
            let id = &self.endpoints[index].id;
            let shift = if unit == dragged {
                // The header rests where the folded list put it; it paints
                // where the pointer holds it.
                let previous = drag.pinned(id);
                let at = window_top(scroll, row)
                    .filter(|_| measured)
                    .map_or(0., |top| {
                        f32::from(drag.pointer.y - drag.grab - top) + previous
                    });
                drag.pin(id, at, now);
                at
            } else {
                let (at, moving) = drag.slide(id, previewed[unit], now);
                plan.sliding |= moving;
                at
            };
            plan.shifts.insert(index, (px(shift), unit == dragged));
            plan.drop.rows.push((unit, row, px(shift)));
            plan.drop.ids.push(id.clone());
        }
        Some(plan)
    }

    /// A host header row in the spaces list at `row`: pressing it may lift
    /// the device, and while one is lifted it takes its place in the plan.
    pub(super) fn place_host_row(
        &self,
        list: Stateful<Div>,
        header: Stateful<Div>,
        (index, row): (usize, usize),
        plan: Option<&DevicePlan>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (shift, carried) = plan.map_or_else(Default::default, |plan| plan.shift(index));
        let header = header
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    if event.click_count == 1 {
                        this.press_device(index, row, event.position, cx);
                    }
                }),
            )
            .when(shift != px(0.), |header| header.top(shift))
            .when(carried, |header| header.cursor_grabbing());
        if carried {
            // Painted last so it floats over the headers it passes.
            list.child(deferred(header).with_priority(1))
        } else {
            list.child(header)
        }
    }

    /// Follows the pointer, resolving a lifted header's gap against `drop`.
    /// Returns whether the move belongs to the drag.
    pub(super) fn move_device_drag(
        &mut self,
        position: Point<Pixels>,
        held: bool,
        drop: Option<&DeviceDrop>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(drag) = self.device_drag.as_mut() else {
            return false;
        };
        // A release outside the window never reaches the sidebar.
        if !held {
            let lifted = drag.lifted;
            self.device_drag = None;
            cx.notify();
            return lifted;
        }
        drag.pointer = position;
        let moved = position - drag.origin;
        if !drag.lifted && f32::from(moved.x.abs().max(moved.y.abs())) > LIFT_DISTANCE {
            self.lift_device(cx);
        }
        let scroll = self.sidebar_scroll[0].clone();
        let Some(drag) = self.device_drag.as_mut().filter(|drag| drag.lifted) else {
            return false;
        };
        drag.target = drop.and_then(|drop| gap(drop, &scroll, position.y - drag.grab));
        cx.notify();
        true
    }

    /// Ends the drag on release, moving the device if it was lifted over a
    /// gap. Returns whether it had lifted, in which case the release is not
    /// a click.
    pub(super) fn release_device_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.device_drag.take() else {
            return false;
        };
        if !drag.lifted {
            return false;
        }
        cx.notify();
        if let Some((_, to)) = &drag.target {
            let to = match to {
                Beside::Before(id) => Beside::Before(id.as_str()),
                Beside::After(id) => Beside::After(id.as_str()),
            };
            self.move_device(&drag.endpoint, to);
        }
        true
    }

    /// Drops a lifted header where it came from, and forgets a press that
    /// has not lifted yet, so its timer cannot lift it later. Returns whether
    /// one was lifted: only then is the key the drag's.
    pub(crate) fn cancel_device_drag(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(drag) = self.device_drag.take() else {
            return false;
        };
        if drag.lifted {
            cx.notify();
        }
        drag.lifted
    }
}

#[cfg(test)]
mod tests;
