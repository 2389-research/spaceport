//! The background monitors the window polls as it ticks: system load,
//! listening ports, and port forwards on the hosts they watch.
// i18n: migrated

use super::{Flash, SheprdWindow};
use gpui::Context;

impl SheprdWindow {
    /// CPU, memory and GPU are sampled for every enabled host, while they
    /// are shown, a Devices overview is open, or a host picker ranks hosts by
    /// them.
    pub(super) fn update_system_load(&mut self) -> bool {
        if !self.config.show_system_load
            && !self.config.gpu_sampled()
            && !self.devices_overview_open()
            && !self.overview_shown()
            && !self.dispatch_sampling()
            && !self.orchestrator_sampling
        {
            return self.system_load.poll(Vec::new(), false);
        }
        let hosts = self.connected_hosts(crate::usage::Host::load_of);
        self.system_load.poll(hosts, self.config.gpu_sampled())
    }

    /// Listening ports are scanned on the same hosts as CPU and memory.
    pub(super) fn update_listening_ports(&mut self) -> bool {
        if !self.config.show_listening_ports {
            self.tunnels.close_all();
            return self.listening_ports.poll(Vec::new());
        }
        let hosts = self.watched_hosts();
        let changed = self.listening_ports.poll(hosts);
        // A tunnel lives as long as its remote port is listed at the address
        // it dials and its SSH child runs; either ending removes its close
        // button.
        let ports = &self.listening_ports;
        let closed = self.tunnels.retain(|key| ports.tunnels_to(key));
        changed || closed
    }

    /// The machines background monitors watch: this one always, a remote
    /// host while it is connected, so a dropped host is not dialled every
    /// few seconds.
    pub(super) fn watched_hosts(&self) -> Vec<crate::usage::Host> {
        self.connected_hosts(crate::usage::Host::of)
    }

    /// The hosts `host` finds for the enabled endpoints `watched_hosts` would
    /// watch. Only connected hosts are sampled by a monitor.
    fn connected_hosts(
        &self,
        host: fn(&sheprd_client::ConnectTarget) -> Option<crate::usage::Host>,
    ) -> Vec<crate::usage::Host> {
        self.endpoints
            .iter()
            .enumerate()
            .filter(|(index, endpoint)| {
                let live = if *index == self.selected_endpoint {
                    &self.live
                } else {
                    &endpoint.live
                };
                endpoint.enabled
                    && (live.status.is_connected() || !endpoint.connection.target.is_remote())
            })
            .filter_map(|(_, endpoint)| host(&endpoint.connection.target))
            .collect()
    }

    /// Forwards outlive a dropped connection, since SSH may still reach the
    /// host, but end once their host is removed or disabled. A report the
    /// user did not just ask for is flashed.
    pub(crate) fn update_port_forwards(&mut self, cx: &mut Context<Self>) {
        let endpoints = &self.endpoints;
        let mut changed = self.port_forwards.retain_hosts(|target| {
            endpoints.iter().any(|endpoint| {
                endpoint.enabled
                    && endpoint
                        .saved_ssh()
                        .is_some_and(|(saved, _)| saved == target)
            })
        });
        let notices = self.port_forwards.poll();
        if let Some(notice) = notices.last() {
            let flash = match notice {
                crate::port_forward::Notice::Listening { .. } => Flash::success(notice.text()),
                crate::port_forward::Notice::Ended { .. } => Flash::warning(notice.text()),
            };
            self.show_flash(flash, cx);
            changed = true;
        }
        if changed {
            cx.notify();
        }
    }

    /// The machine the selected endpoint runs on, for its load.
    pub(crate) fn selected_host(&self) -> Option<crate::usage::Host> {
        self.endpoints
            .get(self.selected_endpoint)
            .and_then(|endpoint| crate::usage::Host::load_of(&endpoint.connection.target))
    }
}
