// ABOUTME: Resolves answer citations only against observation replies in the same turn.
// ABOUTME: Keeps source identity opaque and labels independent of navigation authority.
use super::*;
use sheprd_chat_protocol::{EntryId, Transcript};
use std::collections::HashMap;

pub(super) fn reading_text(text: &str) -> String {
    let mut shown = String::new();
    let mut rest = text;
    while let Some(link) = rest.find("](spaceport-source:") {
        let Some(start) = rest[..link].rfind('[') else {
            break;
        };
        let Some(end) = rest[link..].find(')') else {
            break;
        };
        shown.push_str(&rest[..start]);
        shown.push_str(&rest[start + 1..link]);
        rest = &rest[link + end + 1..];
    }
    shown.push_str(rest);
    shown
}

pub(super) fn answers(transcript: &Transcript) -> HashMap<EntryId, Vec<(Source, u64)>> {
    let mut answers = HashMap::new();
    for turn in transcript
        .entries
        .split_inclusive(|entry| matches!(entry.kind, EntryKind::User { .. }))
    {
        // Each user entry closes the preceding turn; it carries no observation evidence.
        let mut known = HashMap::new();
        for entry in turn {
            if let EntryKind::Tool(tool) = &entry.kind {
                for content in &tool.content {
                    if let ToolContent::Text { text } = content
                        && let Ok(reply) = serde_json::from_str::<observation::Reply>(text)
                    {
                        let sources = match reply {
                            observation::Reply::Sessions {
                                sessions,
                                observed_at_ms,
                                ..
                            } => sessions
                                .into_iter()
                                .map(|session| (session.source, observed_at_ms))
                                .collect(),
                            observation::Reply::Excerpt {
                                source,
                                observed_at_ms,
                                ..
                            } => vec![(*source, observed_at_ms)],
                            observation::Reply::Unavailable { .. } => Vec::new(),
                        };
                        for (source, observed) in sources {
                            known.insert(source.target.0.clone(), (source, observed));
                        }
                    }
                }
            }
        }
        for entry in turn {
            if let EntryKind::Agent { text } = &entry.kind {
                let mut resolved = Vec::new();
                for suffix in text
                    .split("spaceport-source:")
                    .skip(1)
                    .take(observation::MAX_SESSIONS)
                {
                    let target = suffix
                        .split(|c: char| !c.is_ascii_alphanumeric() && c != '-')
                        .next()
                        .unwrap_or("");
                    if let Some(source) = known.get(target)
                        && !resolved.contains(source)
                    {
                        resolved.push(source.clone());
                    }
                }
                if !resolved.is_empty() {
                    answers.insert(entry.id, resolved);
                }
            }
        }
    }
    answers
}

#[cfg(test)]
mod tests;
