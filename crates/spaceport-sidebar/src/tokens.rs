//! Resolve sidebar tokens, omitting missing values and empty rows.
//!
//! The daemon's `[sidebar]` config names what a row shows; this reads it into
//! the tokens a layout paints. Rules on the status token match
//! [`status_key`], never the word a row shows, so a rule keeps working in
//! every language; the word itself is the app's, passed in as `status_word`.

use crate::{
    agents::{agent_place, state_label, status_key},
    metrics::segment_budgets,
};
use gpui::SharedString;
use spaceport_client::{
    protocol::{AgentStatus, ClientShellAgent, ClientShellSnapshot},
    text::agent_names,
};
use spaceport_settings::{AgentLayout, AgentToken, Rows, SpaceLayout, SpaceToken, TokenStyle};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedToken {
    pub kind: TokenKind,
    pub style: TokenStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    StateIcon,
    Text(SharedString, TextRole),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextRole {
    Directory,
    Status,
    Workspace,
    Agent,
    Secondary,
    Muted,
}

impl TokenKind {
    pub fn text(&self) -> Option<&SharedString> {
        match self {
            Self::Text(text, _) => Some(text),
            _ => None,
        }
    }
}

fn token_values(tokens: &[(String, String)]) -> HashMap<&str, &str> {
    tokens
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect()
}

fn shared_text<'a>(strings: &mut HashMap<&'a str, SharedString>, text: &'a str) -> SharedString {
    strings
        .entry(text)
        .or_insert_with(|| SharedString::from(text))
        .clone()
}

/// Resolves each configured token. Rules on the status token match `state`,
/// the daemon's name for the status, so a rule keeps working in every
/// language.
fn resolve_rows<T>(
    rows: &Rows<T>,
    state: &str,
    mut value: impl FnMut(&T) -> Option<TokenKind>,
) -> Vec<Vec<ResolvedToken>> {
    rows.iter()
        .filter_map(|row| {
            let resolved: Vec<_> = row
                .iter()
                .filter_map(|configured| {
                    let kind = value(&configured.token)?;
                    let style = match &kind {
                        TokenKind::Text(_, TextRole::Status) => configured.style_for(state)?,
                        TokenKind::Text(text, _) => configured.style_for(text)?,
                        _ => configured.style,
                    };
                    Some(ResolvedToken { kind, style })
                })
                .collect();
            (!resolved.is_empty()).then_some(resolved)
        })
        .collect()
}

/// The rows for one agent. `machine` is the host label when more than one
/// endpoint is shown. An agent whose workspace has gone has no rows.
/// `status_word` is the app's word for a status, which a `state_text` token
/// shows unless the agent's integration named its state.
pub fn agent_rows(
    layout: &AgentLayout,
    agent: &ClientShellAgent,
    snapshot: &ClientShellSnapshot,
    machine: Option<&str>,
    status_word: impl Fn(AgentStatus) -> String,
) -> Option<Vec<Vec<ResolvedToken>>> {
    let (workspace, tab) = agent_place(agent, snapshot)?;
    let pane = agent
        .title
        .as_deref()
        .and_then(spaceport_client::text::caption)
        .or_else(|| {
            snapshot
                .panes
                .iter()
                .find(|pane| pane.pane_id == agent.pane_id)
                .and_then(|pane| pane.label.as_deref())
        });
    let status = match agent.agent_status {
        AgentStatus::Unknown => AgentStatus::Idle,
        status => status,
    };
    let fallback = status_word(status);
    let state_text = state_label(agent, &fallback);
    let state_rule = state_label(agent, status_key(status));
    let tokens = token_values(&agent.tokens);
    let mut strings = HashMap::new();
    let mut text = |value, role| TokenKind::Text(shared_text(&mut strings, value), role);
    let mut rows = resolve_rows(
        layout.rows_for(agent.agent.as_deref()),
        &state_rule,
        |token| {
            Some(match token {
                AgentToken::StateIcon => TokenKind::StateIcon,
                AgentToken::StateText => text(&state_text, TextRole::Status),
                AgentToken::Machine => text(machine?, TextRole::Secondary),
                AgentToken::Workspace => text(workspace, TextRole::Workspace),
                AgentToken::Tab => text(tab?, TextRole::Secondary),
                AgentToken::Pane => text(pane?, TextRole::Secondary),
                AgentToken::Agent => text(
                    agent_names(agent).into_iter().flatten().next()?,
                    TextRole::Agent,
                ),
                AgentToken::TerminalTitle => {
                    text(agent.terminal_title.as_deref()?, TextRole::Muted)
                }
                AgentToken::TerminalTitleStripped => {
                    text(agent.terminal_title_stripped.as_deref()?, TextRole::Muted)
                }
                AgentToken::Custom(name) => text(tokens.get(name.as_str())?, TextRole::Muted),
            })
        },
    );
    if rows.is_empty() {
        rows.push(vec![ResolvedToken {
            kind: TokenKind::StateIcon,
            style: TokenStyle::default(),
        }]);
    }
    Some(rows)
}

pub struct SpaceContext<'a> {
    pub label: &'a str,
    pub branch: Option<&'a str>,
    pub status: AgentStatus,
    pub tokens: &'a [(String, String)],
    pub indented: bool,
}

/// A workspace's rows, as [`agent_rows`] reads an agent's.
pub fn space_rows(
    layout: &SpaceLayout,
    context: SpaceContext<'_>,
    status_word: impl Fn(AgentStatus) -> String,
) -> Vec<Vec<ResolvedToken>> {
    let state = status_word(context.status);
    let mut strings = HashMap::new();
    let mut text = |value, role| TokenKind::Text(shared_text(&mut strings, value), role);
    let tokens = token_values(context.tokens);
    let mut rows = resolve_rows(&layout.rows, status_key(context.status), |token| {
        Some(match token {
            SpaceToken::StateIcon => TokenKind::StateIcon,
            SpaceToken::StateText => text(&state, TextRole::Status),
            SpaceToken::Workspace => text(context.label, TextRole::Workspace),
            SpaceToken::Branch if context.indented => return None,
            SpaceToken::Branch => text(context.branch?, TextRole::Secondary),
            SpaceToken::Custom(name) => text(tokens.get(name.as_str())?, TextRole::Muted),
        })
    });
    if rows.is_empty() {
        rows.push(Vec::new());
    }
    rows
}

/// Text tokens join with a middle dot; a leading state icon takes one space.
pub fn separator(previous: &ResolvedToken, _current: &ResolvedToken) -> &'static str {
    if matches!(previous.kind, TokenKind::StateIcon) {
        " "
    } else {
        " \u{b7} "
    }
}

/// Glyph budget for each token of a row that must fit `available` glyphs,
/// or `None` for a token dropped entirely. Fixed-width icons always stay.
/// When even one glyph per text token does not
/// fit, text tokens are dropped from the left until the rest fit, then the
/// survivors grow a glyph at a time in turn.
pub fn budgets(
    row: &[ResolvedToken],
    fixed_width: impl Fn(&TokenKind) -> usize,
    available: usize,
) -> Vec<Option<usize>> {
    let fixed: Vec<usize> = row.iter().map(|token| fixed_width(&token.kind)).collect();
    let flexible: Vec<usize> = row
        .iter()
        .map(|token| token.kind.text().map_or(0, |text| text.width()))
        .collect();
    let minimum = |active: &[bool]| -> usize {
        (0..row.len())
            .filter(|index| active[*index])
            .fold((0, None), |(total, previous), index| {
                let gap =
                    previous.map_or(0, |prev| separator(&row[prev], &row[index]).chars().count());
                (
                    total + fixed[index] + usize::from(flexible[index] > 0) + gap,
                    Some(index),
                )
            })
            .0
    };
    let mut active = vec![true; row.len()];
    if minimum(&active) > available {
        for (index, width) in flexible.iter().enumerate() {
            if *width > 0 {
                active[index] = false;
            }
        }
        for index in (0..row.len()).rev() {
            if flexible[index] == 0 {
                continue;
            }
            active[index] = true;
            if minimum(&active) > available {
                active[index] = false;
            }
        }
    }
    let lengths: Vec<_> = flexible
        .iter()
        .zip(&active)
        .map(|(width, active)| if *active { *width } else { 0 })
        .collect();
    let minimum_text: usize = lengths.iter().map(|width| usize::from(*width > 0)).sum();
    let reserved = minimum(&active).saturating_sub(minimum_text);
    let budgets = segment_budgets(&lengths, available.saturating_sub(reserved));
    (0..row.len())
        .map(|index| active[index].then_some(budgets[index].max(fixed[index])))
        .collect()
}

#[cfg(test)]
mod tests;
