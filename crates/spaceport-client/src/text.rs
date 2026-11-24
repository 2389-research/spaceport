//! Checks for untrusted text shown in the app.

use crate::protocol::{ClientShellAgent, ClientShellSnapshot, ClientShellTab};

/// What an agent is called wherever it is listed. The fallback, for an
/// agent that reports no name at all, is the daemon's identity word and
/// stays English while other areas borrow the name as `&str`.
pub fn agent_name(agent: &ClientShellAgent) -> &str {
    agent_names(agent)
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|name| !name.is_empty())
        .unwrap_or("agent")
}

pub fn agent_names(agent: &ClientShellAgent) -> [Option<&str>; 4] {
    [
        agent.display_agent.as_deref(),
        agent.name.as_deref(),
        agent.agent.as_deref(),
        agent.title.as_deref().and_then(caption),
    ]
}

/// Strip only a leading agent paste envelope, never ordinary markup inside a
/// title. A wrapper-only/truncated transport marker is not a useful caption.
pub fn caption(text: &str) -> Option<&str> {
    let original = text;
    let text = text.trim();
    if text.is_empty() || text == "</pasted_content>" {
        return None;
    }
    let Some(rest) = text.strip_prefix("<pasted_content") else {
        return Some(original);
    };
    if rest
        .chars()
        .next()
        .is_some_and(|c| c != '>' && !c.is_ascii_whitespace())
    {
        return Some(original);
    }
    let end = text
        .as_bytes()
        .iter()
        .take(512)
        .position(|byte| *byte == b'>')?;
    let body = text[end + 1..].trim();
    let body = body
        .strip_suffix("</pasted_content>")
        .unwrap_or(body)
        .trim();
    (!body.is_empty()).then_some(body)
}

/// A tab's display label, with a useful fallback for an agent transport marker.
/// This does not mutate the authoritative snapshot or a user's labels.
pub fn tab_label<'a>(snapshot: &'a ClientShellSnapshot, tab: &'a ClientShellTab) -> &'a str {
    caption(&tab.label)
        .or_else(|| {
            snapshot
                .panes
                .iter()
                .filter(|pane| pane.tab_id == tab.tab_id)
                .find_map(|pane| pane.label.as_deref().and_then(caption))
        })
        .or_else(|| {
            snapshot
                .agents
                .iter()
                .filter(|agent| agent.tab_id == tab.tab_id)
                .find_map(|agent| agent.title.as_deref().and_then(caption))
        })
        .or_else(|| {
            snapshot
                .workspaces
                .iter()
                .find(|workspace| workspace.workspace_id == tab.workspace_id)
                .and_then(|workspace| caption(&workspace.label))
        })
        .unwrap_or("Terminal")
}

/// Bytes of clipboard payload this client will produce. Terminal content is
/// untrusted and one cell's symbol carries as many bytes as it likes, so a
/// copy of it — or of the transcript drawn above it — is bounded rather than
/// trusted to be screen-sized.
pub const MAX_COPY_BYTES: usize = 4 << 20;

/// A control or direction-override character, which untrusted text loses.
pub fn unsafe_char(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}' | '\u{061c}' | '\u{2028}' | '\u{2029}')
}

/// `text` cut to `limit` characters, without the characters `unsafe_char`
/// refuses.
pub fn safe_text(text: &str, limit: usize) -> String {
    // Bound scanning as well as output, even for a payload made entirely of controls.
    text.chars()
        .take(limit)
        .filter(|c| !unsafe_char(*c))
        .collect()
}

#[cfg(test)]
mod tests;
