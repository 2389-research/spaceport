//! The device table: each host and its reported agents.
//! Opening a row lists agents; choosing an agent shows its exact pane.

use super::{Device, Link, view::Look};
use crate::i18n::fl;
use crate::{
    SheprdWindow,
    browser::{Slot, TabId},
};
use gpui::{prelude::*, *};
use sheprd_client::protocol::AgentStatus;
use std::collections::HashSet;

const DEVICE_MIN: f32 = 180.;
/// Every column but the device's, which takes the rest, by the message
/// heading it; the last, the row's chevron, has none.
pub(super) const COLUMNS: &[(&str, f32)] = &[("devices-overview-column-agents", 230.), ("", 12.)];

/// A column's heading in the shown language.
fn heading(id: &str) -> String {
    crate::i18n::message(id, &[])
}

/// Where an open row's agents start: under the device's name, past its dot.
const AGENT_INDENT: f32 = 31.;

/// Rows and headings wrap the agent count together at narrow widths.
fn cells(device: impl IntoElement, agents: impl IntoElement, chevron: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_x_3()
        .gap_y_2()
        .child(div().flex_1().min_w(px(DEVICE_MIN)).child(device))
        .child(div().flex_none().w(px(COLUMNS[0].1)).child(agents))
        .child(div().flex_none().w(px(COLUMNS[1].1)).child(chevron))
}

pub(super) fn devices(
    look: &Look,
    shown: &[&Device],
    slot: Slot,
    tab: TabId,
    expanded: &HashSet<String>,
    cx: &mut Context<SheprdWindow>,
) -> Div {
    let theme = look.theme;
    let header = cells(
        fl!("devices-overview-column-device"),
        heading(COLUMNS[0].0),
        div(),
    )
    .px_3()
    .py_2()
    .border_b_1()
    .border_color(rgb(theme.active))
    .text_size(look.small())
    .text_color(rgb(theme.subtext()));
    let rows = shown.iter().enumerate().map(|(position, device)| {
        let open = expanded.contains(&device.id);
        let id = device.id.clone();
        let row_id = device.id.clone();
        let row = cells(
            name(look, device),
            agents(look, device),
            svg()
                .size(px(COLUMNS[1].1))
                .path(if open {
                    "icons/chevron-down.svg"
                } else {
                    "icons/chevron-right.svg"
                })
                .text_color(rgb(theme.muted)),
        )
        .id(SharedString::from(
            slot.selector(&format!("devices-row-{}", device.id)),
        ))
        .debug_selector(move || slot.selector(&format!("devices-row-{row_id}")))
        .px_3()
        .py_2()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme.active)))
        .on_click(cx.listener(move |this, _, _, cx| {
            if let Some(page) = this.devices_overview.pages.get_mut(&tab) {
                if !page.expanded.remove(&id) {
                    page.expanded.insert(id.clone());
                }
                cx.notify();
            }
        }));
        div()
            .flex()
            .flex_col()
            .when(position > 0, |unit| {
                unit.border_t_1().border_color(rgb(theme.active))
            })
            .child(row)
            .when(open, |unit| unit.child(agent_list(look, device, slot, cx)))
    });
    look.panel()
        .debug_selector(move || slot.selector("devices-table"))
        .bg(rgb(theme.background))
        .flex()
        .flex_col()
        .child(header)
        .children(rows)
        .when(shown.is_empty(), |table| {
            table.child(
                div()
                    .p_4()
                    .text_color(rgb(theme.muted))
                    .child(fl!("devices-overview-no-match")),
            )
        })
}

/// An open row's agents, working first; clicking one shows its pane.
fn agent_list(look: &Look, device: &Device, slot: Slot, cx: &mut Context<SheprdWindow>) -> Div {
    let theme = look.theme;
    let list = div()
        .debug_selector({
            let id = device.id.clone();
            move || slot.selector(&format!("devices-agents-{id}"))
        })
        .flex()
        .flex_col()
        .pb_2()
        .bg(rgb(theme.surface));
    if device.agents.is_empty() {
        return list.child(
            div()
                .pl(px(AGENT_INDENT))
                .pt_2()
                .text_color(rgb(theme.muted))
                .child(match device.link {
                    Link::Online => fl!("devices-overview-device-no-agents"),
                    _ => fl!("devices-overview-device-unknown"),
                }),
        );
    }
    list.children(device.agents.iter().map(|agent| {
        let (endpoint, pane) = (device.id.clone(), agent.pane_id.clone());
        let (tab, workspace) = (agent.tab_id.clone(), agent.workspace_id.clone());
        div()
            .id(SharedString::from(slot.selector(&format!(
                "devices-agent-{}-{}",
                device.id, agent.pane_id
            ))))
            .debug_selector({
                let id = format!("devices-agent-{}-{}", device.id, agent.pane_id);
                move || slot.selector(&id)
            })
            .flex()
            .items_center()
            .gap_2()
            .pl(px(AGENT_INDENT))
            .pr_3()
            .py_1()
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme.active)))
            .child(look.dot(agent.status))
            .child(
                crate::icons::AgentIcon::from_identity(agent.identity.as_deref())
                    .element(13., theme.foreground),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(120.))
                    .truncate()
                    .child(agent.name.clone()),
            )
            .child(
                look.mono(agent.workspace.clone())
                    .flex_1()
                    .min_w_0()
                    .truncate(),
            )
            .child(
                div()
                    .flex_none()
                    .text_size(look.small())
                    .text_color(rgb(look.status(agent.status)))
                    .child(crate::sidebar::status_text(agent.status)),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                // Read before navigating, which may switch devices.
                let on_screen = this.endpoints[this.selected_endpoint].id == endpoint
                    && this
                        .browser_key()
                        .is_some_and(|(_, shown)| shown == workspace);
                this.navigate_endpoint(&endpoint, crate::NavigationTarget::Pane(&pane), cx);
                // The window leaves a page for its terminal when the daemon's
                // focus moves to another tab or workspace. An agent in the
                // tab already focused moves nothing, so show it here.
                if on_screen {
                    this.terminal_focus_moved(&tab, false, cx);
                }
                window.focus(&this.focus, cx);
            }))
    }))
}

fn name(look: &Look, device: &Device) -> Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .min_w_0()
        .child(look.link(device.link))
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .child(
                    div()
                        .truncate()
                        .font_weight(FontWeight::MEDIUM)
                        .child(device.label.clone()),
                )
                .child(look.mono(device.address.clone()).truncate()),
        )
}

/// A dot per agent, then `2/5 working`, then how many wait for input.
fn agents(look: &Look, device: &Device) -> Div {
    let theme = look.theme;
    let tally = device.tally;
    let text = match device.link {
        Link::Offline => fl!("devices-overview-offline"),
        Link::Connecting => fl!("devices-overview-connecting"),
        Link::Disabled => fl!("devices-overview-disabled"),
        Link::Online if tally.total() == 0 => fl!("devices-overview-no-agents"),
        Link::Online => fl!(
            "devices-overview-agents-working",
            working = tally.working,
            total = tally.total()
        ),
    };
    let blocked = look.status(AgentStatus::Blocked);
    div()
        .flex()
        .items_center()
        .gap_2()
        .min_w_0()
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(3.))
                .children(device.dots.iter().map(|status| look.dot(*status))),
        )
        .child(look.mono(text).min_w_0().truncate())
        .when(tally.blocked > 0, |row| {
            row.child(
                div()
                    .flex_none()
                    .px(px(5.))
                    .rounded(px(crate::config::corners::SMALL))
                    .bg(rgb(crate::config::mix(theme.background, blocked, 18)))
                    .text_color(rgb(blocked))
                    .text_size(look.small())
                    .child(fl!(
                        "devices-overview-agents-blocked",
                        count = tally.blocked
                    )),
            )
        })
}
