//! The Overview host column shows reported agents, connectivity and ports.
//! Host cards and the column fold without changing their runtime targets.

use crate::{
    SheprdWindow,
    devices_overview::{Device, Link, Look, Tally},
    i18n::fl,
};
use gpui::{prelude::*, *};

const WIDTH: f32 = 290.;
const STRIP: f32 = 40.;
/// Ports a host card lists; the count says the rest.
const PORTS: usize = 6;

impl SheprdWindow {
    pub(super) fn render_overview_hosts(&self, look: &Look, cx: &mut Context<Self>) -> AnyElement {
        let devices: Vec<Device> = self
            .overview_devices()
            .into_iter()
            .filter(|device| device.link != Link::Disabled)
            .collect();
        let theme = look.theme;
        let toggle = div()
            .id("overview-hosts-toggle")
            .debug_selector(|| "overview-hosts-toggle".into())
            .size(px(24.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(crate::config::corners::CONTROL))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.active)))
            .child(
                svg()
                    .path("icons/panel-right.svg")
                    .size(px(14.))
                    .text_color(rgb(theme.muted)),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                this.overview.hosts_closed = Some(!this.overview_hosts_closed());
                cx.notify();
            }));
        if self.overview_hosts_closed() {
            return div()
                .debug_selector(|| "overview-hosts-strip".into())
                .w(px(STRIP))
                .flex_none()
                .pt_3()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .border_l_1()
                .border_color(rgb(theme.active))
                .child(toggle)
                .children(devices.iter().map(|device| {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_1()
                        .child(
                            svg()
                                .path("icons/computer.svg")
                                .size(px(14.))
                                .text_color(rgb(theme.subtext())),
                        )
                        .child(look.link(device.link))
                }))
                .into_any_element();
        }
        div()
            .id("overview-hosts")
            .debug_selector(|| "overview-hosts".into())
            .w(px(WIDTH))
            .flex_none()
            .h_full()
            .overflow_y_scroll()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .border_l_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(look.small())
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("overview-hosts"))
                    .child(
                        div()
                            .text_color(rgb(theme.muted))
                            .child(devices.len().to_string()),
                    )
                    .child(div().flex_1())
                    .child(toggle),
            )
            .children(
                devices
                    .iter()
                    .map(|device| self.host_card(look, device, cx)),
            )
            .into_any_element()
    }

    fn host_card(&self, look: &Look, device: &Device, cx: &mut Context<Self>) -> Div {
        let theme = look.theme;
        let folded = self.overview.folded.contains(&device.id);
        let id = device.id.clone();
        let agents = match device.link {
            Link::Online => agents_line(device.tally),
            Link::Connecting => fl!("overview-connecting"),
            Link::Offline | Link::Disabled => fl!("overview-not-connected"),
        };
        let header = div()
            .id(SharedString::from(format!("overview-host-{}", device.id)))
            .debug_selector({
                let id = format!("overview-host-{}", device.id);
                move || id.clone()
            })
            .p_2()
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(crate::config::corners::PANEL))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.active)))
            .child(
                svg()
                    .path(if folded {
                        "icons/chevron-right.svg"
                    } else {
                        "icons/chevron-down.svg"
                    })
                    .size(px(10.))
                    .flex_none()
                    .text_color(rgb(theme.muted)),
            )
            .child(look.link(device.link))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(device.label.clone()),
            )
            .when(folded, |header| {
                let total = device.tally.total();
                header.when(total > 0, |header| {
                    header.child(look.mono(total.to_string()))
                })
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                if !this.overview.folded.remove(&id) {
                    this.overview.folded.insert(id.clone());
                }
                cx.notify();
            }));
        let ports = self.host_ports(device);
        let card = format!("overview-host-card-{}", device.id);
        div()
            .debug_selector(move || card.clone())
            .flex()
            .flex_col()
            .rounded(px(crate::config::corners::PANEL))
            .bg(rgb(theme.surface))
            .child(header)
            .when(!folded, |card| {
                card.child(
                    div()
                        .px_2()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_size(look.small())
                                .text_color(rgb(theme.muted))
                                .child(agents),
                        )
                        .when(!ports.is_empty(), |card| {
                            let more = ports.len().saturating_sub(PORTS);
                            card.child(
                                div()
                                    .pt_1()
                                    .flex()
                                    .flex_wrap()
                                    .gap_1()
                                    .children(ports.into_iter().take(PORTS).map(|port| {
                                        div()
                                            .px(px(5.))
                                            .rounded(px(crate::config::corners::SMALL))
                                            .border_1()
                                            .border_color(rgb(theme.active))
                                            .child(look.mono(port))
                                    }))
                                    .when(more > 0, |ports| {
                                        ports.child(look.mono(format!("+{more}")))
                                    }),
                            )
                        }),
                )
            })
    }

    /// `:3000 vite` for each port the host's workspaces listen on, lowest
    /// first, when ports are scanned.
    fn host_ports(&self, device: &Device) -> Vec<String> {
        let Some(endpoint) = self.endpoints.get(device.index) else {
            return Vec::new();
        };
        let live = if device.index == self.selected_endpoint {
            &self.live
        } else {
            &endpoint.live
        };
        let (Some(daemon), Some(snapshot)) = (
            crate::listening_ports::Daemon::of(&endpoint.connection.target),
            live.snapshot.as_deref(),
        ) else {
            return Vec::new();
        };
        let mut ports: Vec<(u16, String)> = snapshot
            .workspaces
            .iter()
            .filter_map(|workspace| self.listening_ports.get(&daemon, &workspace.workspace_id))
            .flat_map(|listed| listed.ports.iter())
            .map(|port| (port.number, port.process.clone()))
            .collect();
        ports.sort();
        ports.dedup();
        ports
            .into_iter()
            .map(|(number, process)| format!(":{number} {process}"))
            .collect()
    }
}

/// An online host's agents, leaving out counts of none as the rest of the
/// Overview does.
pub(super) fn agents_line(tally: Tally) -> String {
    match tally.working {
        0 => fl!("overview-agents", total = tally.total()),
        working => fl!(
            "overview-agents-working",
            total = tally.total(),
            working = working
        ),
    }
}
