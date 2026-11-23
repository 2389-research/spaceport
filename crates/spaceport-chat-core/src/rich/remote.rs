//! File links in a conversation from another host. Their paths name files
//! there, so they are shown as inert text with the path written out after
//! the label; the drawn text and the selectable text both come from here,
//! so a copy matches what is on screen byte for byte.

use super::{Inline, Style};
use std::borrow::Cow;

/// `content` with each file link made inert and its target appended once;
/// `is_local_path` is the app's test for a destination that names a file on
/// the machine this client runs on.
pub fn remote_content(content: &[Inline], is_local_path: fn(&str) -> bool) -> Cow<'_, [Inline]> {
    if !content.iter().any(|part| {
        matches!(part, Inline::Text(_, style)
            if style.link.as_deref().is_some_and(is_local_path))
    }) {
        return Cow::Borrowed(content);
    }
    let mut shown = Vec::new();
    for (index, part) in content.iter().enumerate() {
        let Inline::Text(text, style) = part else {
            shown.push(part.clone());
            continue;
        };
        let Some(target) = style.link.as_deref().filter(|url| is_local_path(url)) else {
            shown.push(part.clone());
            continue;
        };
        let mut inert = style.clone();
        inert.link = None;
        shown.push(Inline::Text(text.clone(), inert));
        // Emphasis/code can split one label into several styled text spans.
        // Keep their formatting, then show its destination once at the end.
        if !matches!(content.get(index + 1), Some(Inline::Text(_, next))
            if next.link.as_deref() == Some(target))
        {
            shown.push(Inline::Text(format!(" ({target})"), Style::default()));
        }
    }
    Cow::Owned(shown)
}
