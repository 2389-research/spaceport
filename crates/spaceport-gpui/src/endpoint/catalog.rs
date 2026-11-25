//! Reconciling the window's endpoints and selection with the saved-host
//! catalog the crate loads: this module rebuilds the window's remote endpoints
//! from the devices that catalog describes, and hands back the stored choice
//! once the endpoint it names is serving.
use super::{Endpoint, LOCAL};
use crate::SpaceportWindow;
use gpui::Context;
use spaceport_client::{SavedHost, WslHost};
use spaceport_endpoint::{devices, entry_changed};

impl SpaceportWindow {
    pub(super) fn restore_selection(&mut self, cx: &mut Context<Self>) {
        if !self.catalog.restore_pending {
            return;
        }
        let Some(id) = self.catalog.desired.clone() else {
            return;
        };
        if self
            .endpoints
            .iter()
            .any(|endpoint| endpoint.id == id && endpoint.enabled() && endpoint.serving())
        {
            // One handoff attempt: activation failure may fall back to Local,
            // but must neither overwrite the preference nor loop on every tick.
            self.catalog.restore_pending = false;
            let _ = self.switch_endpoint(&id, cx);
        }
    }

    /// SSH hosts and WSL distributions only; for fixtures.
    #[cfg(test)]
    pub(crate) fn reconcile_catalog(
        &mut self,
        hosts: Vec<SavedHost>,
        wsl: Vec<WslHost>,
        cx: &mut Context<Self>,
    ) {
        self.reconcile_devices(hosts, wsl, cx);
    }

    /// Replace the remote endpoints with every saved SSH and WSL device.
    pub(super) fn reconcile_devices(
        &mut self,
        hosts: Vec<SavedHost>,
        wsl: Vec<WslHost>,
        cx: &mut Context<Self>,
    ) {
        let mut devices = devices(hosts, wsl);
        devices.sort_by_key(|device| super::order::rank(&self.device_order, &device.id));
        let selected = &self.endpoints[self.selected_endpoint];
        let selected_id = selected.id.clone();
        let selected_retired = self.selected_endpoint != 0
            && !devices.iter().any(|device| {
                device.id == selected_id
                    && device.enabled
                    && !entry_changed(selected.saved_target(), &device.target)
            });
        if selected_retired {
            let _ = self.switch_endpoint(LOCAL, cx);
        }
        let selected_id = self.endpoints[self.selected_endpoint].id.clone();
        for endpoint in &self.endpoints[1..] {
            if !devices.iter().any(|device| device.id == endpoint.id) {
                tracing::info!(
                    category = "catalog",
                    id = endpoint.id,
                    "Device left the catalog"
                );
            }
        }
        for device in &devices {
            if !self
                .endpoints
                .iter()
                .any(|endpoint| endpoint.id == device.id)
            {
                tracing::info!(
                    category = "catalog",
                    id = device.id,
                    enabled = device.enabled,
                    "Device joined the catalog"
                );
            }
        }
        let mut previous = std::mem::take(&mut self.endpoints);
        let mut next = vec![previous.remove(0)];
        for device in devices {
            let mut endpoint = if let Some(index) = previous.iter().position(|e| e.id == device.id)
            {
                previous.remove(index)
            } else {
                Endpoint::new(
                    device.id,
                    device.label.clone(),
                    device.target.clone(),
                    device.enabled,
                )
            };
            let changed = endpoint.enabled() != device.enabled
                || entry_changed(endpoint.saved_target(), &device.target);
            if changed {
                endpoint.retarget(device.target.clone(), device.enabled);
            }
            endpoint.label = device.label;
            endpoint.saved = Some(device.target);
            next.push(endpoint);
        }
        self.endpoints = next;
        self.prune_device_removals();
        self.selected_endpoint = self
            .endpoints
            .iter()
            .position(|e| e.id == selected_id)
            .unwrap_or(0);
        // Devices picked for the sidebar that left or were disabled are no
        // longer listed; the selected one was followed above.
        self.device_filter.reconcile(
            self.endpoints
                .iter()
                .filter(|endpoint| endpoint.enabled())
                .map(|endpoint| endpoint.id.as_str()),
        );
        // A tab torn off another window may belong to a host just listed.
        self.land_arrival(true, cx);
        cx.notify();
    }
}

#[cfg(test)]
mod tests;
