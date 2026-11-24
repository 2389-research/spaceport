//! The device order the window lists and arranges, by the user's own: the
//! crate's `rank` and `moved` decide where a device sorts and lands, and this
//! module keeps the endpoints in that order and remembers it.
//!
//! Much of the window knows Local as endpoint 0, so the endpoints keep it
//! first and the rest in the user's order; the sidebar places Local among
//! them where the user put it, and lists it first until then.

pub(super) use spaceport_endpoint::{moved, rank};

use super::{Endpoint, LOCAL};
use crate::SpaceportWindow;
use spaceport_reorder::Beside;

impl SpaceportWindow {
    /// Every device with its endpoint index, in the order the sidebar lists
    /// them.
    pub(crate) fn displayed_devices(&self) -> Vec<(usize, &Endpoint)> {
        let mut devices: Vec<_> = self.endpoints.iter().enumerate().collect();
        // The rest are already arranged, so Local goes before the first one
        // the user put after it.
        if let Some(local) = self.device_order.iter().position(|id| id == LOCAL) {
            let at = devices[1..]
                .iter()
                .position(|(_, endpoint)| rank(&self.device_order, &endpoint.id) > local)
                .map_or(devices.len(), |at| at + 1);
            devices[..at].rotate_left(1);
        }
        // Grouped by tag, each section keeping the user's order.
        if self.device_tags.any() {
            let all = self.device_tags.all();
            let filter = self.device_tag_filter.as_deref();
            devices.sort_by_key(|(_, endpoint)| self.device_tags.rank(&endpoint.id, filter, &all));
        }
        devices
    }

    /// Lists the devices in the user's order, keeping the selection on the
    /// device it was on.
    pub(crate) fn arrange_devices(&mut self) {
        let selected = self.endpoints[self.selected_endpoint].id.clone();
        let order = &self.device_order;
        self.endpoints[1..].sort_by_key(|endpoint| rank(order, &endpoint.id));
        self.selected_endpoint = self
            .endpoints
            .iter()
            .position(|endpoint| endpoint.id == selected)
            .unwrap_or(0);
    }

    /// Moves device `id` beside another and remembers the new order. Returns
    /// whether anything moved.
    pub(crate) fn move_device(&mut self, id: &str, to: Beside<&str>) -> bool {
        let ids: Vec<_> = self
            .displayed_devices()
            .into_iter()
            .map(|(_, endpoint)| endpoint.id.as_str())
            .collect();
        let Some(order) = moved(&ids, id, to) else {
            return false;
        };
        if order.iter().map(String::as_str).eq(ids.iter().copied()) {
            return false;
        }
        self.device_order = order;
        self.arrange_devices();
        self.save_chrome();
        true
    }
}

#[cfg(test)]
mod tests;
