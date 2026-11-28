//! One host's workspace rows, and the lines that ride under each of them.

use super::SpacesRows;
use crate::{
    NavigationTarget, SpaceportWindow,
    direction::Logical,
    sidebar::{
        Plan, STATUS_WIDTH,
        agents::status_text,
        cell::{Cell, Fold, RowContext, RowData, WorkspaceRow},
        line_height,
        row::{RowIcon, RowLift, RowTree},
        tokens::{self, SpaceContext},
        visible_workspace_entries,
        workspaces::{
            displayed_workspace_status, workspace_badge, workspace_label, workspace_note,
        },
    },
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::ClientShellSnapshot;
use spaceport_reorder::preview;

/// One listed host's workspace rows: the host they belong to, the snapshot they
/// read, and the two daemon keys their mutes and listening ports answer to —
/// the one thing a row needs that the host's view does not carry, since only
/// the connection's target names a daemon.
pub(super) struct HostRows<'a> {
    pub(super) view: crate::endpoint::HostView<'a>,
    pub(super) index: usize,
    pub(super) selected: bool,
    pub(super) snapshot: &'a ClientShellSnapshot,
    pub(super) mutes: crate::notification_mutes::Daemon,
    pub(super) ports: Option<crate::listening_ports::Daemon>,
}

impl SpaceportWindow {
    /// Appends `host`'s workspace rows to `spaces`, in list order, each with
    /// the note, ports and nested agents that ride under it, and returns the
    /// list with the key of the focused nested agent among them, for the
    /// reveal.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 390 lines")]
    pub(super) fn append_workspace_rows(
        &self,
        mut spaces: SpacesRows,
        host: HostRows<'_>,
        row_cx: &RowContext<'_>,
        m: &super::super::measure::SidebarMeasure<'_>,
        cx: &mut Context<Self>,
    ) -> (SpacesRows, Option<SharedString>) {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let layout = m.look.density;
        let content_x = m.content_x;
        let rows = m.row_layout;
        let menu_target = m.menu_target;
        let worktree_glyph = crate::worktree_identity::glyph(&self.config.worktrees, theme);
        let HostRows {
            view,
            index: endpoint_index,
            selected,
            snapshot,
            mutes,
            ports,
        } = host;
        let endpoint_id = view.id.to_owned();
        let silenced = crate::notification_mutes::Mutes::silenced_in(cx, &mutes, snapshot);
        let collapsed_repos = view.collapsed_repos;
        let entries = visible_workspace_entries(&snapshot.workspaces, collapsed_repos);
        let drag = self
            .workspace_drag
            .as_ref()
            .filter(|drag| selected && drag.previewing(&snapshot.workspaces));
        let floating = drag.is_some_and(|drag| drag.floating());
        let plan = drag.and_then(|drag| Plan::new(&snapshot.workspaces, &drag.workspace));
        // Each unit's shift for the drop in preview, from the heights the last
        // frame laid out: shifting moves rows, never resizes them.
        let base = spaces.rows;
        let shifts = plan.as_ref().map_or_else(Vec::new, |plan| {
            spaces.drop_requests = (0..=plan.unit_count())
                .map(|slot| plan.request(&snapshot.workspaces, slot))
                .collect();
            spaces.drop_dragged = plan.dragged();
            let mut heights = vec![0.; plan.unit_count()];
            for (position, entry) in entries.iter().enumerate() {
                if let (Some(unit), Some(bounds)) = (
                    plan.unit_of(entry.0),
                    self.sidebar_scroll[0].bounds_for_item(base + position),
                ) {
                    heights[unit] += f32::from(bounds.size.height);
                    if m.spaces_custom && !entry.1 {
                        heights[unit] += f32::from(self.config.sidebar_layout.spaces.row_gap)
                            * line_height(font);
                    }
                }
            }
            let slot = drag
                .and_then(|drag| drag.target())
                .map_or(plan.dragged(), |target| target.slot);
            preview(&heights, plan.dragged(), slot)
        });
        // A child closes the group when no child follows it.
        let closes: Vec<bool> = (0..entries.len())
            .map(|position| {
                entries[position].1 && !entries.get(position + 1).is_some_and(|next| next.1)
            })
            .collect();
        // Gap separates one repository from the next, not a parent from the
        // linked worktrees under it. Reset per host.
        let mut previous_indented = None;
        let mut nested_focus = None;
        for (position, (index, indented, group)) in entries.into_iter().enumerate() {
            if self.host_folded(&view) {
                break;
            }
            let workspace = &snapshot.workspaces[index];
            let linked = crate::worktree_identity::is_linked(workspace);
            if selected && workspace.focused {
                spaces.highlighted = Some(spaces.rows);
            }
            let unit = plan.as_ref().and_then(|plan| plan.unit_of(index));
            // The lifted row and the rows it carries, such as its group's
            // children, follow the pointer together while it floats.
            let carried = unit.is_some_and(|unit| unit == spaces.drop_dragged) && floating;
            let shift = match (drag, unit) {
                (Some(drag), Some(_)) if carried => {
                    drag.pin(&workspace.workspace_id, f32::from(drag.offset()), m.now);
                    drag.offset()
                }
                (Some(drag), Some(unit)) => {
                    let (at, moving) = drag.slide(&workspace.workspace_id, shifts[unit], m.now);
                    spaces.sliding |= moving;
                    px(at)
                }
                _ => px(0.),
            };
            if let Some(unit) = unit {
                spaces.drop_rows.push((unit, spaces.rows, shift));
            }
            spaces.rows += 1;
            let id = workspace.workspace_id.clone();
            let press_id = id.clone();
            let context_id = id.clone();
            let hover_id = id.clone();
            let context_endpoint = endpoint_id.clone();
            let navigate_endpoint = endpoint_id.clone();
            let collapse_endpoint = endpoint_id.clone();
            let grouped = group.is_some() || indented;
            let tree = match (indented, closes[position]) {
                (false, _) => RowTree::None,
                (true, false) => RowTree::Child,
                (true, true) => RowTree::LastChild,
            };
            let fold_toggle = group.clone().map(|key| (endpoint_id.clone(), key));
            let fold = group.map(|key| Fold {
                id: SharedString::from(format!("collapse-{endpoint_id}-{id}")).into(),
                index,
                collapsed: collapsed_repos.contains(&key),
                toggle: Box::new(cx.listener(move |this, _, _, cx| {
                    this.toggle_repository(&collapse_endpoint, &key, cx);
                })),
            });
            let folds = fold.as_ref().map(|fold| fold.collapsed).zip(fold_toggle);
            let label = workspace_label(workspace, indented);
            let shown_status = if self.config.usage.inline {
                displayed_workspace_status(&snapshot.workspaces, workspace, collapsed_repos)
            } else {
                workspace.agent_status
            };
            let muted = silenced.workspace(&workspace.workspace_id);
            let lines = if m.spaces_custom {
                tokens::space_rows(
                    &self.config.sidebar_layout.spaces,
                    SpaceContext {
                        label,
                        branch: workspace.branch.as_deref(),
                        status: shown_status,
                        tokens: &workspace.tokens,
                        indented,
                    },
                    status_text,
                )
            } else {
                Vec::new()
            };
            let gap = if m.spaces_custom {
                match previous_indented.replace(indented) {
                    Some(_) if !indented => {
                        f32::from(self.config.sidebar_layout.spaces.row_gap) * line_height(font)
                    }
                    _ => 0.,
                }
            } else {
                0.
            };
            // Lines under the row start under its label column, clear of the
            // status dot.
            let under_left = content_x
                + row_cx.nest
                + if indented {
                    layout.child_indent() + row_cx.indicators.width(font) - STATUS_WIDTH
                } else {
                    0.
                }
                + row_cx.indicators.width(font)
                + layout.gap();
            let note = workspace_note(workspace, crate::worktree_notes::Notes::of(cx), view.id)
                .map(SharedString::from);
            let note_line = note.clone().map(|note| {
                let endpoint = endpoint_id.clone();
                let workspace = id.clone();
                crate::sidebar::note_line::element(
                    &format!("{endpoint_id}-{id}"),
                    note,
                    (under_left, content_x),
                    font,
                    theme,
                )
                .on_click(cx.listener(
                    move |this, event: &ClickEvent, window, cx| {
                        cx.stop_propagation();
                        this.edit_worktree_note(
                            &endpoint,
                            &workspace,
                            event.position(),
                            window,
                            cx,
                        );
                    },
                ))
            });
            // A child's trunk runs on past its note and ports to the next
            // sibling, as it runs through the row itself.
            let trunk = rows.trunk_below(&format!("{endpoint_id}-{id}"), tree, row_cx);
            let ports = self
                .config
                .show_listening_ports
                .then_some(ports.as_ref())
                .flatten()
                .and_then(|ports| self.listening_ports.get(ports, &workspace.workspace_id))
                .map(|listed| {
                    div()
                        .debug_selector(|| format!("ports-{endpoint_id}-{id}"))
                        .h(px(line_height(font)))
                        .flex_none()
                        .w_full()
                        .min_w_0()
                        .ps(px(under_left))
                        .pe(px(content_x))
                        .text_size(px((font.size * 0.85).round()))
                        .child(crate::listening_ports::chips(
                            listed,
                            (&endpoint_id, &workspace.workspace_id),
                            &self.tunnels,
                            theme,
                            (font.size * 0.85).round(),
                            cx,
                        ))
                });
            let element = Cell::new(
                rows,
                RowData::Workspace(WorkspaceRow {
                    workspace,
                    label,
                    tree,
                    // A child hangs from the main checkout on the guide lines;
                    // one standing alone has no line to hang from, so it takes
                    // the branch glyph where the repository icon sits.
                    icon: if let Some(glyph) = worktree_glyph.filter(|_| linked && !indented) {
                        RowIcon::Worktree(glyph)
                    } else if indented || !self.config.show_repository_icons {
                        RowIcon::None
                    } else if workspace.branch.is_none() {
                        // Keep the icon slot aligned with checkout rows.
                        RowIcon::Blank
                    } else {
                        RowIcon::Repository
                    },
                    fold,
                    grouped,
                    badge: workspace_badge((note.as_deref(), muted)),
                    removing: false,
                    status: shown_status,
                    lines,
                }),
                row_cx,
            )
            .selected(selected && workspace.focused)
            .highlighted(selected && menu_target == Some(id.as_str()))
            .lift(match (carried, floating) {
                (true, _) => RowLift::Lifted,
                (false, true) => RowLift::Passed,
                (false, false) => RowLift::Resting,
            })
            .row()
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
            .when(
                self.menu.page == Some(crate::menu::Page::Workspace),
                |row| {
                    let view = cx.entity().downgrade();
                    let endpoint = endpoint_id.clone();
                    let workspace = id.clone();
                    row.child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                let bounds = bounds.intersect(&window.content_mask().bounds);
                                window.on_mouse_event(
                                    move |event: &MouseDownEvent, phase, window, cx| {
                                        // The overlay dismisses first in bubble order. Use
                                        // clipped row geometry because it occludes our hitbox.
                                        if phase == DispatchPhase::Bubble
                                            && event.button == MouseButton::Right
                                            && bounds.contains(&event.position)
                                        {
                                            let _ = view.update(cx, |this, cx| {
                                                cx.stop_propagation();
                                                if this.navigate_endpoint(
                                                    &endpoint,
                                                    NavigationTarget::Workspace(&workspace),
                                                    cx,
                                                ) {
                                                    this.open_workspace_menu(
                                                        &workspace,
                                                        event.position,
                                                        window,
                                                        cx,
                                                    );
                                                    this.menu.opening_right_click = this.menu.page
                                                        == Some(crate::menu::Page::Workspace);
                                                }
                                            });
                                        }
                                    },
                                );
                            },
                        )
                        .absolute()
                        .inset_0()
                        .size_full(),
                    )
                },
            )
            .id(SharedString::from(format!("workspace-{endpoint_id}-{id}")))
            .map(|row| {
                let name = crate::sidebar::accessibility::workspace_name(
                    label,
                    linked,
                    shown_status,
                    muted,
                    row_cx.host,
                );
                self.workspace_item(row, name, selected && workspace.focused, folds, cx)
            })
            .when(m.multi, |row| {
                row.debug_selector(|| format!("workspace-{endpoint_id}-{id}"))
            })
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                let selected = this.navigate_endpoint(
                    &navigate_endpoint,
                    NavigationTarget::Workspace(&id),
                    cx,
                );
                window.focus(&this.focus, cx);
                // The first click of a double-click has already selected the
                // row, so the second only opens the rename dialog its menu's
                // Rename opens, which takes focus. The fold chevron stops its
                // clicks, so folding fast never renames.
                if selected && event.click_count() == 2 {
                    this.open_workspace_dialog_for(
                        &id,
                        crate::menu::WorkspaceAction::Rename,
                        event.position(),
                        window,
                        cx,
                    );
                }
            }))
            // Only the selected endpoint's rows arm the hover menu: another
            // endpoint's menu would have to select it first, and resting the
            // pointer must not switch which daemon is shown. The feature is
            // opt-in, so rows stay unarmed without it.
            .when(selected && self.config.features.sidebar_hover_menu, |row| {
                row.on_hover(cx.listener(move |this, hovered: &bool, window, _| {
                    this.hover_workspace(&hover_id, *hovered, window);
                }))
            })
            // Holding a press lifts the row for reordering. Another endpoint's
            // rows would have to select it first.
            .when(selected, |row| {
                row.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                        if this.config.features.sidebar_hover_menu {
                            this.press_hovered_workspace(&press_id, event.position);
                        }
                        if event.click_count == 1 {
                            this.press_workspace(&press_id, event.position, cx);
                        }
                    }),
                )
            })
            .when(carried, |row| row.cursor_grabbing());
            let indent = layout.child_indent() * if indented { 2. } else { 1. };
            let agent_rows = if m.nested_agents {
                let (rows, focused) = self.nested_agent_rows(
                    endpoint_index,
                    snapshot,
                    &workspace.workspace_id,
                    (row_cx, indent),
                    cx,
                );
                nested_focus = nested_focus.or(focused);
                rows
            } else {
                Vec::new()
            };
            // The note, ports and nested agents ride under their row as one
            // list item, so the drop preview still measures one height per
            // workspace.
            let element = if note_line.is_none() && ports.is_none() && agent_rows.is_empty() {
                element
                    .when(gap > 0., |row| row.mt(px(gap)))
                    .when(shift != px(0.), |row| row.top(shift))
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .w_full()
                    .relative()
                    .when(gap > 0., |unit| unit.mt(px(gap)))
                    .when(shift != px(0.), |unit| unit.top(shift))
                    .child(element)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .relative()
                            .children(note_line)
                            .children(ports)
                            .children(agent_rows)
                            // Last, so a selected nested agent's fill does not
                            // hide it, as a row draws its guide over its own
                            // fill.
                            .children(trunk),
                    )
                    .into_any_element()
            };
            let list = spaces.list;
            spaces.list = if carried {
                // Painted last so it floats over the rows it passes, while its
                // layout slot keeps the others' positions stable.
                list.child(deferred(element).with_priority(1))
            } else {
                list.child(element)
            };
        }
        (spaces, nested_focus)
    }
}
