//! Drawing the collapsed sidebar's rail: what it measured, its spaces and
//! agents lists, and the frame around them.

use super::{
    AGENT_ICON, Columns, HOST_MARK, Hint, RowState, SidebarMode, agent_name, agent_status_word,
    aligned_row, describe, dot_slot, host_chrome, host_header, host_initial, host_status,
    indicator, label_text, mark_slot, muted_word, rail_button, rail_columns, rail_font,
    status_word, visible_workspace_entries,
};
use crate::direction::Logical;
use crate::i18n::fl;
use crate::notification_mutes::{Daemon, Mutes, Silenced};
use crate::sidebar::{
    agents::{Indicators, agent_place},
    layout::{self, SidebarLook},
    line_height,
    workspaces::workspace_label,
};
use crate::{Command, NavigationTarget, SpaceportWindow, config::FontConfig, fonts::StyledFont};
use gpui::{prelude::*, *};
use spaceport_client::protocol::ClientShellAgent;
use std::collections::HashMap;

impl SpaceportWindow {
    /// The rail in three phases: what it measured, its spaces and agents
    /// lists, and the frame around them.
    pub(crate) fn render_rail(
        &self,
        indicators: Indicators,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let measured = self.rail_measure(indicators, window, cx);
        let spaces = self.rail_spaces(&measured, cx);
        let agents = self.rail_agents(&measured, cx);
        self.rail_frame(&measured, spaces, agents, cx)
    }

    /// What the rail measures before it draws: its width, the face it fits
    /// the columns into, and what the mutes silence.
    fn rail_measure<'a>(
        &'a self,
        indicators: Indicators,
        window: &Window,
        cx: &'a App,
    ) -> RailMeasure<'a> {
        let width = SidebarMode::Rail
            .width(None, f32::from(window.viewport_size().width))
            .unwrap_or(0.);
        let look = layout::for_config(&self.config, self.fill().mark());
        let hint = Hint::new(&self.theme);
        let multi = self.endpoints.len() > 1;
        // What each listed host's mutes silence. A muted workspace shows its
        // mark in the icon column, which the rail then keeps for every row.
        let mutes = Mutes::any(cx);
        let silenced: HashMap<usize, Silenced<'_>> = self
            .displayed_devices()
            .into_iter()
            .filter_map(|(index, endpoint)| {
                let live = self.live_of(index);
                let snapshot = live.snapshot.as_deref()?;
                Some((index, mutes?.silenced(&Daemon::of(endpoint), snapshot)))
            })
            .collect();
        let slot = multi || silenced.values().any(Silenced::any_workspace);
        // A large sidebar font would make the columns wider than the rail and
        // clip the status icons, so the rail steps the font down until they fit.
        let font = rail_font(&self.config.sidebar, indicators, multi, slot);
        let columns = rail_columns(&font, indicators, multi, slot);
        let split = self.sidebar_split.unwrap_or(0.5).clamp(0.1, 0.9);
        RailMeasure {
            columns,
            font,
            hint,
            indicators,
            look,
            mutes,
            multi,
            silenced,
            split,
            width,
        }
    }

    /// The spaces list: every listed host's header box, then its workspaces.
    fn rail_spaces(&self, measured: &RailMeasure<'_>, cx: &Context<Self>) -> Stateful<Div> {
        let mut spaces = div()
            .id("rail-spaces")
            .debug_selector(|| "rail-spaces".into())
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        for (endpoint_index, endpoint, live) in self.listed_hosts() {
            let host = endpoint.view();
            if measured.multi {
                spaces = spaces.child(self.rail_host_box(measured, endpoint_index, &host, cx));
            }
            if self.host_folded(&host) {
                continue;
            }
            let Some(snapshot) = &live.snapshot else {
                continue;
            };
            for (index, indented, _) in
                visible_workspace_entries(&snapshot.workspaces, host.collapsed_repos)
            {
                spaces = spaces.child(self.rail_workspace_row(
                    measured,
                    endpoint_index,
                    &host,
                    &snapshot.workspaces[index],
                    indented,
                    cx,
                ));
            }
        }
        spaces
    }

    /// A host's letter, the box that heads its workspaces, and the menu and
    /// click that act on it.
    fn rail_host_box(
        &self,
        measured: &RailMeasure<'_>,
        endpoint_index: usize,
        host: &spaceport_endpoint::HostView<'_>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let RailMeasure {
            font,
            hint,
            look,
            mutes,
            width,
            ..
        } = measured;
        let theme = &self.theme;
        let selected = endpoint_index == self.selected_endpoint;
        let endpoint_id = host.id.to_owned();
        let initial = host_initial(host.label);
        let select_id = endpoint_id.clone();
        let menu_id = endpoint_id.clone();
        let connected = host.status.is_connected();
        let text = if host.collapsed {
            fl!("sidebar-rail-collapsed", host = host.label)
        } else {
            host.label.to_owned()
        };
        // An offline or folded host has no rows to carry the mark.
        let text = muted_word(
            Some(text.clone()),
            mutes.is_some_and(|mutes| mutes.host(host.id)),
        )
        .unwrap_or(text);
        let box_id = endpoint_id.clone();
        let chrome = host_chrome(selected, *look, theme);
        host_header(font, *look)
            .child(
                div()
                    .debug_selector(move || format!("rail-host-box-{box_id}"))
                    .relative()
                    .size(px(HOST_MARK))
                    .flex_none()
                    .row()
                    .items_center()
                    .justify_center()
                    .rounded(px(chrome.radius))
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.fill)
                    .text_color(chrome.letter)
                    .child(label_text(&initial))
                    .child(host_status(
                        format!("rail-host-status-{endpoint_id}"),
                        if connected {
                            crate::menu::online(theme)
                        } else {
                            theme.muted
                        },
                        theme.sidebar_background(),
                        *width,
                    )),
            )
            .id(SharedString::from(format!("rail-host-{endpoint_id}")))
            .debug_selector(|| format!("rail-host-{endpoint_id}"))
            .cursor_pointer()
            .tooltip(hint.with(text))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    cx.stop_propagation();
                    this.open_host_menu(&menu_id, event.position, window, cx);
                    this.menu.opening_right_click = this.menu.page == Some(crate::menu::Page::Host);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                // A refusal shows in the status bar; a click
                // has nothing else to say.
                let _ = this.select_endpoint(&select_id, cx);
                window.focus(&this.focus, cx);
            }))
    }

    /// One workspace's mark, icon, and status dot, with the menu and click
    /// that navigate to it.
    fn rail_workspace_row(
        &self,
        measured: &RailMeasure<'_>,
        endpoint_index: usize,
        host: &spaceport_endpoint::HostView<'_>,
        workspace: &spaceport_client::protocol::ClientShellWorkspace,
        indented: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let RailMeasure {
            columns,
            font,
            hint,
            indicators,
            look,
            multi,
            silenced,
            ..
        } = measured;
        let theme = &self.theme;
        let columns = *columns;
        let selected = endpoint_index == self.selected_endpoint;
        let endpoint_id = host.id.to_owned();
        let host_label = (*multi).then_some(host.label);
        let id = workspace.workspace_id.clone();
        let key = format!("rail-workspace-{endpoint_id}-{id}");
        let navigate_endpoint = endpoint_id.clone();
        let context_endpoint = endpoint_id.clone();
        let context_id = id.clone();
        let focused = selected && workspace.focused;
        let number = workspace.number.to_string();
        let muted = silenced
            .get(&endpoint_index)
            .is_some_and(|silenced| silenced.host() || silenced.workspace(&workspace.workspace_id));
        let marks = div()
            .row()
            .items_center()
            .gap(px(columns.gap))
            .child(mark_slot(
                format!("rail-mark-{key}"),
                columns.badge,
                columns.height,
                if focused {
                    theme.foreground
                } else {
                    theme.muted
                },
                &number,
            ))
            .when_some(columns.icon, |marks, icon| {
                marks.child(
                    div()
                        .flex_none()
                        .w(px(icon))
                        .h(px(columns.height))
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(muted, |slot| {
                            // The rail's rows name nothing for a screen
                            // reader, so the mark names itself.
                            slot.debug_selector(|| format!("rail-muted-{key}")).child(
                                crate::icons::spoken_muted(
                                    SharedString::from(format!("rail-muted-mark-{key}")),
                                    crate::icons::icon(
                                        crate::icons::MUTED,
                                        icon - 2.,
                                        theme.subtext(),
                                    ),
                                ),
                            )
                        }),
                )
            })
            .child(dot_slot(
                format!("rail-dot-{key}"),
                columns.dot,
                columns.height,
                indicator(workspace.agent_status, font, *indicators),
            ));
        aligned_row(
            &key,
            RowState {
                selected: focused,
                ..RowState::default()
            },
            font,
            *look,
            theme,
            columns,
            marks,
        )
        .id(SharedString::from(key.clone()))
        .debug_selector(|| key)
        .cursor_pointer()
        .tooltip(hint.with(describe(
            host_label,
            workspace_label(workspace, indented),
            None,
            muted_word(status_word(workspace.agent_status), muted).as_deref(),
        )))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                if this.navigate_endpoint(
                    &context_endpoint,
                    NavigationTarget::Workspace(&context_id),
                    cx,
                ) {
                    this.open_workspace_menu(&context_id, event.position, window, cx);
                    this.menu.opening_right_click =
                        this.menu.page == Some(crate::menu::Page::Workspace);
                }
            }),
        )
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Workspace(&id), cx);
            window.focus(&this.focus, cx);
        }))
    }

    /// The agents list: every listed host's agents, after every host's
    /// workspaces, as the panel interleaves them.
    fn rail_agents(&self, measured: &RailMeasure<'_>, cx: &Context<Self>) -> Stateful<Div> {
        let mut agents = div()
            .id("rail-agents")
            .debug_selector(|| "rail-agents".into())
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        let panel = if self.config.show_agents {
            self.panel_agents()
        } else {
            Vec::new()
        };
        for (index, agent) in panel {
            let Some(endpoint) = self.endpoints.get(index) else {
                continue;
            };
            let host = endpoint.view();
            // A collapsed host folds its agents away with its workspaces.
            if self.host_folded(&host) {
                continue;
            }
            let selected = index == self.selected_endpoint;
            let live = self.live_of(index);
            let Some(snapshot) = live.snapshot.as_deref() else {
                continue;
            };
            agents =
                agents.child(self.rail_agent_row(measured, agent, &host, snapshot, selected, cx));
        }
        agents
    }

    /// One agent's mark and status dot, with the click that navigates to it.
    fn rail_agent_row(
        &self,
        measured: &RailMeasure<'_>,
        agent: &ClientShellAgent,
        host: &spaceport_endpoint::HostView<'_>,
        snapshot: &spaceport_client::protocol::ClientShellSnapshot,
        selected: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let RailMeasure {
            columns,
            font,
            hint,
            indicators,
            look,
            multi,
            ..
        } = measured;
        let theme = &self.theme;
        let columns = *columns;
        let endpoint_id = host.id.to_owned();
        let host_label = (*multi).then_some(host.label);
        let id = agent.pane_id.clone();
        let key = format!("rail-agent-{endpoint_id}-{id}");
        let navigate_endpoint = endpoint_id.clone();
        let focused = selected && agent.focused;
        let place = agent_place(agent, snapshot);
        let place = place.map(|(workspace, tab)| match tab {
            Some(tab) => format!("{workspace} / {tab}"),
            None => workspace.to_owned(),
        });
        let icon = AGENT_ICON.min(font.size + 1.);
        let icon_color = if focused {
            theme.foreground
        } else {
            theme.muted
        };
        let agent_icon = || {
            let selector = format!("rail-agent-icon-{endpoint_id}-{id}");
            let mark = crate::icons::AgentIcon::from_identity(agent.agent.as_deref());
            div()
                .debug_selector(move || selector)
                .flex_none()
                .size(px(icon))
                .row()
                .items_center()
                .justify_center()
                .child(mark.sidebar_element(icon, icon_color, theme))
        };
        // The host letter shares the workspace number's column. With one
        // host there is no letter, so the icon takes that column and still
        // starts on the badge's left edge.
        let badge = if *multi {
            mark_slot(
                format!("rail-mark-{key}"),
                columns.badge,
                columns.height,
                theme.muted,
                &host_initial(host.label),
            )
        } else {
            div()
                .debug_selector(|| format!("rail-mark-{key}"))
                .flex_none()
                .w(px(columns.badge))
                .h(px(columns.height))
                .row()
                .items_center()
                .child(agent_icon())
        };
        let marks = div()
            .row()
            .items_center()
            .gap(px(columns.gap))
            .child(badge)
            .when(*multi, |marks| marks.child(agent_icon()))
            // One host whose muted workspaces took the icon column.
            .when_some(columns.icon.filter(|_| !*multi), |marks, icon| {
                marks.child(div().flex_none().w(px(icon)).h(px(columns.height)))
            })
            .child(dot_slot(
                format!("rail-dot-{key}"),
                columns.dot,
                columns.height,
                indicator(agent.agent_status, font, *indicators),
            ));
        aligned_row(
            &key,
            RowState {
                agent: true,
                selected: focused,
                ..RowState::default()
            },
            font,
            *look,
            theme,
            columns,
            marks,
        )
        .when(focused, |row| look.agent_selection(row, theme))
        .id(SharedString::from(key.clone()))
        .debug_selector(|| key)
        .cursor_pointer()
        .tooltip(hint.with(describe(
            host_label,
            agent_name(agent),
            place.as_deref(),
            agent_status_word(agent).as_deref(),
        )))
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Pane(&id), cx);
            window.focus(&this.focus, cx);
        }))
    }

    /// The rail itself: the two lists, the divider between them, and the
    /// button that opens a workspace.
    fn rail_frame(
        &self,
        measured: &RailMeasure<'_>,
        spaces: Stateful<Div>,
        agents: Stateful<Div>,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let RailMeasure {
            font,
            hint,
            look,
            split,
            width,
            ..
        } = measured;
        let theme = &self.theme;
        div()
            .id("sidebar-rail")
            .debug_selector(|| "sidebar-rail".into())
            .relative()
            .w(px(*width))
            .flex_none()
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .flex()
            .flex_col()
            .text_font(font)
            .text_size(px(font.size))
            .line_height(px(line_height(font)))
            .text_color(rgb(theme.foreground))
            .bg(self.fill().panel(theme.sidebar_background()))
            .border_e(px(1.))
            .border_color(rgb(theme.active))
            // No expand control of its own: the sidebar toggle is always on
            // screen, over the rail beside the activity bar, else in the
            // header or leading the tab row.
            .child(
                div()
                    .debug_selector(|| "rail-spaces-section".into())
                    .flex()
                    .flex_col()
                    .flex_1()
                    .map(|mut section| {
                        section.style().flex_grow =
                            Some(if self.config.show_agents { *split } else { 1. });
                        section
                    })
                    .min_h_0()
                    .overflow_hidden()
                    .child(spaces),
            )
            .when(self.config.show_agents, |rail| {
                rail.child(div().flex_none().h(px(1.)).bg(rgb(theme.active)))
                    .child(
                        div()
                            .debug_selector(|| "rail-agents-section".into())
                            .flex()
                            .flex_col()
                            .flex_1()
                            .map(|mut section| {
                                section.style().flex_grow = Some(1. - *split);
                                section
                            })
                            .min_h_0()
                            .overflow_hidden()
                            .child(agents),
                    )
            })
            .child(
                rail_button("rail-new-workspace", font, *look, theme)
                    .text_color(rgb(theme.muted))
                    .hover(|s| s.text_color(rgb(theme.foreground)))
                    .tooltip(hint.with(fl!("sidebar-new-workspace")))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.command(Command::Workspace, window, cx);
                    }))
                    .child(label_text("+")),
            )
    }
}

/// What the rail measured: its width, the face and columns it fits in, and
/// what the mutes silence for each listed host.
struct RailMeasure<'a> {
    columns: Columns,
    font: FontConfig,
    hint: Hint,
    indicators: Indicators,
    look: SidebarLook,
    mutes: Option<&'a Mutes>,
    multi: bool,
    silenced: HashMap<usize, Silenced<'a>>,
    split: f32,
    width: f32,
}
