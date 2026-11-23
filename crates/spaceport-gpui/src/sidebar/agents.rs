//! The agents list: how it is sorted and the words its status is shown in.
//! Status comes from the daemon's snapshot, never from guessing at terminal
//! output. The model the marks and words are made of — the indicator colors and
//! sizes, the daemon's status names, how a place, pane and label are read from
//! a snapshot, and the marks themselves — lives in `spaceport_sidebar::agents`;
//! what is left here is the order the panel paints and the words it shows.
//! i18n: migrated

pub(super) mod sections;

use super::label_text;
use crate::SpaceportWindow;
use crate::i18n::fl;
use crate::preferences::AgentSort;
use gpui::{prelude::*, *};
use spaceport_client::protocol::{AgentStatus, ClientShellAgent};

pub(crate) use spaceport_sidebar::agents::{
    Indicators, agent_labels, agent_place, icon_mark, pane_name, rail, sorted_agents, state_label,
    status_indicator, status_style,
};

pub(super) fn agents_sort(
    window: &SpaceportWindow,
    cx: &mut Context<SpaceportWindow>,
) -> Stateful<Div> {
    let theme = &window.theme;
    let view = window
        .live
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.agent_view_label.clone());
    let label = view
        .clone()
        .unwrap_or_else(|| sort_label(window.agent_sort));
    div()
        .id("agents-sort")
        .debug_selector(|| "agents-sort".into())
        // A plugin's view only names itself; the sort is a button.
        .map(|sort| match &view {
            Some(view) => sort
                .role(Role::Label)
                .aria_value(fl!("sidebar-agents-view", view = view.as_str())),
            None => window.chrome_stop(
                crate::window::chrome_focus::Region::Sidebar,
                sort.role(Role::Button).aria_label(fl!(
                    "sidebar-agents-sort",
                    sort = sort_label(window.agent_sort)
                )),
            ),
        })
        .flex_none()
        .min_w_0()
        .truncate()
        .text_color(rgb(theme.muted))
        .when(view.is_none(), |sort| {
            sort.cursor_pointer()
                .hover(|style| style.text_color(rgb(theme.foreground)))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.agent_sort = this.agent_sort.toggled();
                    this.agent_sort_modified = true;
                    this.save_chrome();
                    cx.notify();
                }))
        })
        .child(label_text(&label))
}

/// The agents header's name for how it sorts, which a click toggles.
fn sort_label(sort: AgentSort) -> String {
    match sort {
        AgentSort::Grouped => fl!("sidebar-sort-grouped"),
        AgentSort::Priority => fl!("sidebar-sort-priority"),
    }
}

/// Every listed host's agents in the order the panel paints them, with the
/// host's endpoint index. Grouped lists one host's agents after another's, in
/// sidebar order. Priority merges hosts as Herdr's terminal client does: a
/// disconnected host's last-known agents after connected ones, then attention,
/// then the most recent change on the clock every connection shares, so a
/// blocked agent on one host rises above an idle one on another. A plugin
/// view orders only its own host's agents, so while any host shows one, each
/// host keeps its own order.
pub(super) fn panel_agents<'a>(
    hosts: impl IntoIterator<Item = (usize, &'a crate::state::LiveState)>,
    sort: AgentSort,
    now: u64,
) -> Vec<(usize, &'a ClientShellAgent)> {
    let mut merge = sort == AgentSort::Priority;
    let mut rows = Vec::new();
    for (index, live) in hosts {
        let Some(snapshot) = live.snapshot.as_deref() else {
            continue;
        };
        merge &= snapshot.agent_view_label.is_none();
        rows.extend(
            sorted_agents(snapshot, sort)
                .into_iter()
                .map(|agent| (index, live, agent)),
        );
    }
    if merge {
        rows.sort_by_cached_key(|(index, live, agent)| sections::order(*index, live, agent, now));
    }
    rows.into_iter()
        .map(|(index, _, agent)| (index, agent))
        .collect()
}

pub(crate) use spaceport_client::text::agent_name;

/// The word the daemon's `state_text` token shows for a status when its
/// sidebar config asks for it, in the app's language. Rules match
/// the daemon's status key instead, so they hold in every language.
pub(crate) fn status_text(status: AgentStatus) -> String {
    match status {
        AgentStatus::Working => fl!("sidebar-status-working"),
        AgentStatus::Blocked => fl!("sidebar-status-blocked"),
        AgentStatus::Done => fl!("sidebar-status-done"),
        AgentStatus::Idle => fl!("sidebar-status-idle"),
        AgentStatus::Unknown => fl!("sidebar-status-unknown"),
    }
}

#[cfg(test)]
mod tests;
