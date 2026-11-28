//! A row's two lines and its one row, painted from the inputs every row of
//! one render shares: the label budget elided against measured glyph widths,
//! the guide that ties a child to its parent, the status slot, and the badge.
//! Every word goes through the app's [`Painters::label`], so its probes see
//! the text a row painted.

use super::{
    Elbow, RowBadge, RowContext, RowIcon, RowKind, RowState, RowTree, agent_icon_size, agent_mark,
    configured::{TokenLook, configured_lines, configured_status, leading_status},
    guide, removing_dot, row_text, tree_gutter,
};
use crate::{
    agents::{Indicators, status_indicator},
    direction::Logical,
    metrics::{
        ARROW_RESERVE, ICON_RESERVE, STATUS_WIDTH, glyph_width, line_height, scaled,
        segment_budgets,
    },
    tokens::ResolvedToken,
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::AgentStatus;
use unicode_width::UnicodeWidthStr;

/// A row's first line: segments joined by upstream's separator, the primary one
/// carrying the row's weight and color while the rest stay muted. Segments are
/// placed at measured offsets rather than flexed, because GPUI 0.2.2 only
/// ellipsizes text whose width its parent already fixed.
pub fn name_line(
    segments: &[(&str, bool)],
    appearance: (u32, FontWeight, u32),
    width: f32,
    cx: &RowContext<'_>,
) -> Div {
    let (font, painters) = (cx.font, cx.painters);
    let (primary, weight, muted) = appearance;
    let glyph = glyph_width(font);
    let separator = 3. * glyph;
    let separators = segments.len().saturating_sub(1);
    let lengths: Vec<usize> = segments.iter().map(|(text, _)| text.width()).collect();
    let available = (width - separators as f32 * separator).max(0.);
    let budgets = segment_budgets(&lengths, (available / glyph).floor() as usize);
    // The last segment takes the rounding remainder, so one segment fills the
    // line exactly as it did before a line could carry several.
    let used: f32 = budgets.iter().map(|budget| *budget as f32 * glyph).sum();
    let slack = (available - used).max(0.);
    let parts =
        segments
            .iter()
            .zip(budgets)
            .enumerate()
            .map(|(index, ((text, is_primary), budget))| {
                let advance = budget as f32 * glyph
                    + if index + 1 == segments.len() {
                        slack
                    } else {
                        0.
                    };
                let cell = div()
                    .truncate()
                    .when(*is_primary, |part| {
                        part.font_weight(weight).text_color(rgb(primary))
                    })
                    .when(!*is_primary, |part| part.text_color(rgb(muted)))
                    .child(painters.label(*text));
                (if index == 0 { "" } else { " · " }, advance, cell)
            });
    place_line(parts, width, cx)
}

pub(super) fn place_line(
    parts: impl IntoIterator<Item = (&'static str, f32, Div)>,
    width: f32,
    cx: &RowContext<'_>,
) -> Div {
    let (font, painters, muted, right_to_left) =
        (cx.font, cx.painters, cx.theme.muted, cx.right_to_left);
    let mut line = div()
        .relative()
        .w(px(width))
        .h(px(line_height(font)))
        .flex_none()
        .overflow_hidden();
    let mut x = 0.;
    for (separator, advance, cell) in parts {
        if !separator.is_empty() {
            let gap = separator.chars().count() as f32 * glyph_width(font);
            line = line.child(
                div()
                    .absolute()
                    .inset_s(right_to_left, px(x))
                    .w(px(gap))
                    .text_color(rgb(muted))
                    .child(painters.label(separator)),
            );
            x += gap;
        }
        line = line.child(cell.absolute().inset_s(right_to_left, px(x)).w(px(advance)));
        x += advance;
    }
    line
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 223 lines")]
pub fn row(
    // Rows are probed by key, not by label: an agent names its workspace, which
    // already names a row of its own.
    key: &str,
    name: &[(&str, bool)],
    detail: &str,
    kind: RowKind,
    status: AgentStatus,
    indicators: Indicators,
    removing: bool,
    state: RowState,
    tree: RowTree,
    reserve_arrow: bool,
    workspace_icon: RowIcon,
    arrow: Option<Stateful<Div>>,
    badge: Option<RowBadge>,
    // The status word the daemon's `state_text` token asks to show, when its
    // sidebar config names it. Painted at the row's trailing edge in the dot's
    // color so a status reads at a glance, not only by hue.
    status_text: Option<&str>,
    lines: &[Vec<ResolvedToken>],
    cx: &RowContext<'_>,
) -> Div {
    let (font, theme, look, width) = (cx.font, cx.theme, cx.look, cx.width);
    let focused = state.selected;
    let layout = look.density;
    let configured = !lines.is_empty();
    let leading_icon = leading_status(lines);
    let show_status = !configured || leading_icon.is_some() || removing;
    let status_text = status_text.filter(|_| !configured);
    let text_lines = if configured {
        let badge_lines = badge.as_ref().map_or(0, RowBadge::lines);
        lines.len().max(badge_lines)
    } else {
        0
    };
    let content_x = look.content_x();
    let gap = layout.gap();
    let vertical_padding = look.row_padding();
    // Content starts below the highlight's edge, which sits half the row
    // spacing in from the row's own.
    let content_top = vertical_padding + look.spacing() / 2.;
    // An agent with nothing to say below its name, such as one nested under
    // the workspace that names its place, takes one line.
    let show_detail = match kind {
        RowKind::Agent(_) => !detail.is_empty(),
        RowKind::Workspace if tree == RowTree::None => layout.workspace_details(),
        RowKind::Workspace => layout.child_details(),
    };
    let (name_color, weight, detail_color) = row_text(kind, focused, theme);
    let icon_reserve = match workspace_icon {
        RowIcon::None => 0.,
        _ => scaled(font, ICON_RESERVE),
    };
    let status_width = indicators.width(font);
    let extra_status_width = status_width - STATUS_WIDTH;
    let indent = cx.nest
        + if tree == RowTree::None {
            0.
        } else {
            layout.child_indent() + extra_status_width
        };
    // Only a drawn chevron takes its column. The badge hugs the row's edge
    // either way, so an empty column would only cut a group member's name.
    let arrow_reserve = if reserve_arrow && arrow.is_some() {
        scaled(font, ARROW_RESERVE)
    } else {
        0.
    };
    let status_gutter = if show_status { status_width + gap } else { 0. };
    let available = (look.content_width(width) - status_gutter - indent - arrow_reserve).max(0.);
    // Narrow sidebars and large fonts can leave less room than a badge needs.
    // Clip its column within the row rather than painting over the terminal.
    let badge_width = badge
        .as_ref()
        .map_or(0., |badge| badge.width(font).min((available - gap).max(0.)));
    let badge_reserve = if badge.is_some() {
        badge_width + gap
    } else {
        0.
    };
    // The status word keeps its own trailing column, so the label yields to it
    // the same way it yields to a badge, and like the badge it is clipped to
    // the room left rather than painting past the row.
    let status_width = status_text.map_or(0., |text| {
        (text.width() as f32 * glyph_width(font))
            .ceil()
            .min((available - badge_reserve - gap).max(0.))
    });
    let status_reserve = if status_text.is_some() {
        status_width + gap
    } else {
        0.
    };
    let status_color = indicators.color(status);
    let label_width = (available - badge_reserve - status_reserve).max(0.);
    let detail_limit = label_width;
    let agent_icon = match kind {
        RowKind::Agent(icon) => Some(icon),
        RowKind::Workspace => None,
    };
    // An orphan's name is on the first line; normal rows name the agent below
    // its location. Reserve the same fixed icon + gap on whichever line owns it.
    let agent_first = agent_icon.filter(|_| detail.is_empty());
    let agent_detail = agent_icon.filter(|_| !detail.is_empty());
    let (agent_size, agent_reserve) = agent_icon_size(font);
    let name_reserve = icon_reserve + agent_first.map_or(0., |_| agent_reserve);
    div()
        .debug_selector(|| format!("row-{key}"))
        .h(px(look.row_height(
            line_height(font)
                * if configured {
                    text_lines as f32
                } else if show_detail {
                    2.
                } else {
                    1.
                },
        )))
        .w_full()
        .min_w_0()
        .flex_none()
        .relative()
        .ps(cx.right_to_left, px(content_x + indent))
        .pe(cx.right_to_left, px(content_x))
        .row(cx.right_to_left)
        .items_start()
        .gap(px(gap))
        .py(px(content_top))
        .cursor_pointer()
        .map(|row| look.mark(row, key, (state, cx.mark), theme))
        // Tree lines run in the indent the row already reserves, so a child is
        // tied to its parent without box-drawing glyphs in the label.
        .children(guide(
            key,
            tree,
            tree_gutter(cx),
            Elbow::At(content_top + line_height(font) / 2.),
            cx,
        ))
        .when(show_status, |row| {
            let centered = indicators.centered();
            row.child(
                if removing {
                    div()
                        .w(px(indicators.width(font)))
                        .mt(px((line_height(font) - scaled(font, STATUS_WIDTH)) / 2.))
                        .flex_none()
                        .row(cx.right_to_left)
                        .justify_center()
                        .child(removing_dot("worktree-removing", font, theme))
                } else if let Some(token) = leading_icon {
                    configured_status(token, status, cx)
                } else {
                    status_indicator(status, font, indicators, cx.painters, cx.right_to_left)
                }
                .debug_selector(|| format!("mark-{key}"))
                .when(centered, |slot| slot.mt_0().self_center()),
            )
        })
        .child(
            div()
                .flex()
                .flex_col()
                // Avoid zero-basis measurement: GPUI 0.2.2 mutates text run
                // lengths when truncating and reuses them on wider measurements.
                .w(px(label_width))
                .flex_none()
                .overflow_hidden()
                .debug_selector(|| format!("column-{key}"))
                .map(|column| {
                    if configured {
                        return configured_lines(
                            column,
                            key,
                            lines,
                            TokenLook {
                                kind,
                                status,
                                focused,
                            },
                            (label_width, detail_limit),
                            workspace_icon,
                            cx,
                        );
                    }
                    column
                        .child(
                            div()
                                .relative()
                                .w(px(label_width))
                                .h(px(line_height(font)))
                                .when_some(agent_first, |line, icon| {
                                    line.child(agent_mark(key, icon, agent_size, name_color, cx))
                                })
                                .when(workspace_icon.draws(), |title| {
                                    title.child(workspace_icon.slot(key, cx))
                                })
                                .child(
                                    name_line(
                                        name,
                                        (name_color, weight, theme.muted),
                                        (label_width - name_reserve).max(0.),
                                        cx,
                                    )
                                    .debug_selector(|| format!("name-{key}"))
                                    .ms(cx.right_to_left, px(name_reserve.min(label_width))),
                                ),
                        )
                        .when(show_detail, |column| {
                            let detail_x =
                                agent_detail.map_or(0., |_| agent_reserve.min(detail_limit));
                            let room = (detail_limit - detail_x).max(0.);
                            column.child(
                                div()
                                    .relative()
                                    .w(px(detail_limit))
                                    .h(px(line_height(font)))
                                    .when_some(agent_detail, |line, icon| {
                                        line.child(agent_mark(
                                            key,
                                            icon,
                                            agent_size,
                                            detail_color,
                                            cx,
                                        ))
                                    })
                                    .child(
                                        div()
                                            .debug_selector(|| format!("detail-{key}"))
                                            .ms(cx.right_to_left, px(detail_x))
                                            .w(px(room))
                                            .truncate()
                                            .text_color(rgb(detail_color))
                                            .child(cx.painters.label(detail)),
                                    ),
                            )
                        })
                }),
        )
        .when_some(status_text, |row, text| {
            row.child(
                div()
                    .debug_selector(|| format!("status-{key}"))
                    .w(px(status_width))
                    .flex_none()
                    .h(px(line_height(font)))
                    .row(cx.right_to_left)
                    .items_center()
                    .overflow_hidden()
                    .text_color(rgb(status_color))
                    .child(
                        div()
                            .w(px(status_width))
                            .truncate()
                            .child(cx.painters.label(text)),
                    ),
            )
        })
        // The collapse column comes first so the badge can hug the row's edge.
        .when_some(arrow, |row, arrow| row.child(arrow))
        .when_some(badge, |row, badge| {
            row.child(badge.element(key, badge_width, font, theme))
        })
}
