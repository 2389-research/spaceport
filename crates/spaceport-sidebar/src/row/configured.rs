//! Rows drawn from the daemon's configured sidebar tokens rather than the
//! built-in name and detail lines.

use super::{
    RowContext, RowIcon, RowKind, agent_icon_size, agent_mark, paint::place_line, row_text,
};
use crate::{
    agents::status_mark,
    direction::Logical,
    metrics::{ICON_RESERVE, glyph_width, line_height, scaled},
    tokens::{ResolvedToken, TextRole, TokenKind, budgets, separator},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::AgentStatus;
use spaceport_settings::{Theme, TokenStyle};
use unicode_width::UnicodeWidthStr;

/// What colors a configured row's tokens: the row they paint and its state.
#[derive(Clone, Copy)]
pub struct TokenLook {
    pub kind: RowKind,
    pub status: AgentStatus,
    pub focused: bool,
}

fn token_appearance(kind: &TokenKind, look: TokenLook, cx: &RowContext<'_>) -> (u32, FontWeight) {
    let theme = cx.theme;
    let (name, weight, secondary) = row_text(look.kind, look.focused, theme);
    match kind {
        TokenKind::StateIcon | TokenKind::Text(_, TextRole::Status) => {
            (cx.indicators.color(look.status), FontWeight::NORMAL)
        }
        TokenKind::Text(_, TextRole::Directory) => (directory_ink(theme), FontWeight::NORMAL),
        TokenKind::Text(_, TextRole::Workspace) => (name, weight),
        TokenKind::Text(_, TextRole::Secondary | TextRole::Agent) => {
            (secondary, FontWeight::NORMAL)
        }
        _ => (theme.muted, FontWeight::NORMAL),
    }
}

fn directory_ink(theme: &Theme) -> u32 {
    spaceport_settings::contrast::ink_on_chrome(
        spaceport_settings::mix(theme.subtext(), theme.palette[4], 20),
        [
            theme.sidebar_background(),
            theme.active,
            crate::layout::agent_selection_background(theme),
        ],
        4.5,
    )
}

/// A token's base color and weight with its configured `fg`, `dim`, and `bold`.
pub fn styled(
    (color, weight): (u32, FontWeight),
    style: TokenStyle,
    theme: &Theme,
) -> (u32, FontWeight) {
    let color = style.fg.unwrap_or(color);
    let color = if style.dim == Some(true) {
        theme.dimmed(color)
    } else {
        color
    };
    let weight = match style.bold {
        Some(true) => FontWeight::BOLD,
        Some(false) => FontWeight::NORMAL,
        None => weight,
    };
    (color, weight)
}

/// Status icons retain their full width when text truncates.
fn fixed_glyphs(kind: &TokenKind, glyph: f32, status_width: f32) -> usize {
    match kind {
        TokenKind::StateIcon => (status_width / glyph).ceil() as usize,
        _ => 0,
    }
}

/// Fixed token widths preserve GPUI's text truncation during layout.
fn token_line(row: &[ResolvedToken], look: TokenLook, width: f32, cx: &RowContext<'_>) -> Div {
    // Herdr places the app icon on its configured line before budgeting text.
    let compact;
    let (row, icon_gap) = if cx.icon_only_agent
        && row
            .iter()
            .any(|token| matches!(token.kind, TokenKind::Text(_, TextRole::Agent)))
    {
        compact = row
            .iter()
            .filter(|token| !matches!(token.kind, TokenKind::Text(_, TextRole::Agent)))
            .cloned()
            .collect::<Vec<_>>();
        (compact.as_slice(), scaled(cx.font, 4.).min(width.max(0.)))
    } else {
        (row, 0.)
    };
    let text_width = (width - icon_gap).max(0.);
    let status = look.status;
    let (font, theme) = (cx.font, cx.theme);
    let glyph = glyph_width(font);
    let budgets = budgets(
        row,
        |kind| fixed_glyphs(kind, glyph, cx.indicators.width(font)),
        (text_width / glyph).floor() as usize,
    );
    let visible: Vec<_> = row
        .iter()
        .zip(budgets)
        .filter_map(|(token, width)| Some((token, width?)))
        .collect();
    let used = visible.iter().map(|(_, width)| width).sum::<usize>()
        + visible
            .windows(2)
            .map(|pair| separator(pair[0].0, pair[1].0).chars().count())
            .sum::<usize>();
    let slack = (text_width - used as f32 * glyph).max(0.);
    let last_text = visible
        .iter()
        .rposition(|(token, _)| token.kind.text().is_some());
    let parts = visible.iter().enumerate().map(|(index, (token, budget))| {
        let gap = if index == 0 {
            ""
        } else {
            separator(visible[index - 1].0, token)
        };
        let advance = *budget as f32 * glyph + if last_text == Some(index) { slack } else { 0. };
        let (color, weight) = styled(token_appearance(&token.kind, look, cx), token.style, theme);
        let cell = div();
        let cell = match &token.kind {
            TokenKind::StateIcon => cell
                .h(px(line_height(font)))
                .flex()
                .justify_center()
                .debug_selector(|| "inline-status-cell".into())
                .child(
                    status_mark(
                        status,
                        font,
                        cx.status_indicators(),
                        color,
                        weight == FontWeight::BOLD,
                        cx.painters,
                        cx.right_to_left,
                    )
                    .debug_selector(|| "inline-status-mark".into()),
                ),
            // Only text the budget cuts short truncates. An emoji's font is wider
            // than two estimated glyphs, so a label that fits by its display
            // width may overhang its cell slightly rather than collapse to "…".
            TokenKind::Text(text, _) if text.width() <= *budget => cell
                .whitespace_nowrap()
                .font_weight(weight)
                .text_color(rgb(color))
                .child(cx.painters.label(text.clone())),
            TokenKind::Text(text, _) => cell
                .truncate()
                .font_weight(weight)
                .text_color(rgb(color))
                .child(cx.painters.label(text.clone())),
        };
        (gap, advance, cell)
    });
    place_line(parts, width, icon_gap, cx)
}

/// The `state_icon` leading a configured row's first line. It takes the row's
/// status slot; a configured row without one shows no status of its own.
pub fn leading_status(lines: &[Vec<ResolvedToken>]) -> Option<&ResolvedToken> {
    lines
        .first()
        .and_then(|line| line.first())
        .filter(|token| matches!(token.kind, TokenKind::StateIcon))
}

/// The color, and whether bold, a leading `state_icon` paints the status in.
pub fn configured_status_style(
    token: &ResolvedToken,
    status: AgentStatus,
    cx: &RowContext<'_>,
) -> (u32, bool) {
    let (color, weight) = styled(
        (cx.indicators.color(status), FontWeight::NORMAL),
        token.style,
        cx.theme,
    );
    (color, weight == FontWeight::BOLD)
}

/// The status mark a leading `state_icon` styles, offset onto the first line.
pub fn configured_status(token: &ResolvedToken, status: AgentStatus, cx: &RowContext<'_>) -> Div {
    let (color, bold) = configured_status_style(token, status, cx);
    status_mark(
        status,
        cx.font,
        cx.status_indicators(),
        color,
        bold,
        cx.painters,
        cx.right_to_left,
    )
}

/// A line's tokens, less the leading `state_icon` the status slot draws.
fn line_tokens(index: usize, line: &[ResolvedToken]) -> &[ResolvedToken] {
    match line.split_first() {
        Some((first, rest)) if index == 0 && matches!(first.kind, TokenKind::StateIcon) => rest,
        _ => line,
    }
}

fn line_selector(key: &str, index: usize) -> String {
    match index {
        0 => format!("name-{key}"),
        1 => format!("detail-{key}"),
        index => format!("line-{key}-{index}"),
    }
}

/// Configured lines stacked at `width`, for layouts that draw their own
/// status and icons beside the text.
pub fn token_column(
    key: &str,
    lines: &[Vec<ResolvedToken>],
    look: TokenLook,
    width: f32,
    cx: &RowContext<'_>,
) -> Div {
    lines.iter().enumerate().fold(
        div().w(px(width)).flex_none().flex().flex_col(),
        |column, (index, line)| {
            column.child(
                token_line(line_tokens(index, line), look, width, cx)
                    .debug_selector(|| line_selector(key, index)),
            )
        },
    )
}

/// The line whose app name or icon identifies the agent.
pub(crate) fn agent_line(lines: &[Vec<ResolvedToken>]) -> Option<usize> {
    lines.iter().position(|line| {
        line.iter()
            .any(|token| matches!(token.kind, TokenKind::Text(_, TextRole::Agent)))
    })
}

pub(super) fn configured_lines(
    mut column: Div,
    key: &str,
    lines: &[Vec<ResolvedToken>],
    look: TokenLook,
    (label_width, detail_limit): (f32, f32),
    mut workspace_icon: RowIcon,
    cx: &RowContext<'_>,
) -> Div {
    let (font, theme) = (cx.font, cx.theme);
    let agent_icon = match look.kind {
        RowKind::Agent(icon) => Some(icon),
        RowKind::Workspace => None,
    };
    let agent_at = agent_line(lines);
    let (agent_size, agent_reserve) = agent_icon_size(font);
    let workspace_reserve = if matches!(workspace_icon, RowIcon::None) {
        0.
    } else {
        scaled(font, ICON_RESERVE)
    };
    for (index, line) in lines.iter().enumerate() {
        // The second line sits beside the badge's counts, so it may have
        // less room than the first.
        let label_width = if index == 1 {
            detail_limit
        } else {
            label_width
        };
        let agent_here = agent_at == Some(index);
        let reserve = if index == 0 { workspace_reserve } else { 0. }
            + if agent_here { agent_reserve } else { 0. };
        let text_width = (label_width - reserve).max(0.);
        let selector = line_selector(key, index);
        let icon_color = line
            .iter()
            .find(|token| matches!(token.kind, TokenKind::Text(_, TextRole::Agent)))
            .map(|token| styled(token_appearance(&token.kind, look, cx), token.style, theme).0)
            .unwrap_or(theme.muted);
        let mut text = div().relative().w(px(label_width)).h(px(line_height(font)));
        if agent_here && let Some(icon) = agent_icon {
            text = text.child(agent_mark(key, icon, agent_size, icon_color, cx));
        }
        if index == 0 && workspace_icon.draws() {
            text = text.child(std::mem::replace(&mut workspace_icon, RowIcon::None).slot(key, cx));
        }
        text = text.child(
            token_line(line_tokens(index, line), look, text_width, cx)
                .debug_selector(|| selector.clone())
                .ms(cx.right_to_left, px(reserve.min(label_width))),
        );
        column = column.child(text);
    }
    column
}

#[cfg(test)]
mod tests;
