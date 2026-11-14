// ABOUTME: Extracts bounded inert text from prepared chat entries and terminal cells.
// ABOUTME: Refuses unfenced surfaces and detached transcripts without starting readers.
use super::{Bounds, Record, SheprdWindow, UnavailableReason};
use crate::state::LiveState;
use sheprd_client::protocol::ClientShellSnapshot;

pub(super) struct Excerpt {
    pub text: String,
    pub next_offset: Option<usize>,
    pub truncated: bool,
}
fn push(text: &mut String, value: &str, limit: usize) -> bool {
    for c in value
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
    {
        if text.len() + c.len_utf8() > limit {
            return false;
        }
        text.push(c);
    }
    true
}

/// Offsets count source items. A byte-cut item is marked truncated and consumed.
pub(super) fn excerpt(items: impl IntoIterator<Item = impl AsRef<str>>, bounds: Bounds) -> Excerpt {
    let mut items = items.into_iter().skip(bounds.offset).peekable();
    let mut text = String::new();
    let mut consumed = 0;
    let mut cut = false;
    while consumed < bounds.limit {
        if items.peek().is_none() {
            break;
        }
        if consumed > 0 && !push(&mut text, "\n", bounds.max_bytes) {
            cut = true;
            break;
        }
        let Some(item) = items.next() else {
            break;
        };
        consumed += 1;
        if !push(&mut text, item.as_ref(), bounds.max_bytes) {
            cut = true;
            break;
        }
    }
    let more = items.peek().is_some();
    Excerpt {
        text,
        next_offset: more.then_some(bounds.offset + consumed),
        truncated: cut || more || bounds.offset > 0,
    }
}

pub(super) fn transcript<'a>(
    view: &'a SheprdWindow,
    record: &Record,
    index: usize,
) -> Result<&'a sheprd_chat_protocol::Transcript, UnavailableReason> {
    if index != view.selected_endpoint {
        return Err(UnavailableReason::NotObserved);
    }
    let chat = view
        .chats
        .pane(&record.identity.pane)
        .ok_or(UnavailableReason::NotObserved)?;
    if !chat.attached() {
        return Err(UnavailableReason::Disconnected);
    }
    let transcript = chat.session.transcript();
    let session = transcript
        .session
        .as_ref()
        .map(|session| session.id.clone());
    if record.identity.chat.as_ref() != Some(&chat.target)
        || record.identity.chat_session != session
        || record.identity.chat_epoch != chat.session.since().map(|cursor| cursor.epoch)
    {
        return Err(UnavailableReason::StaleTarget);
    }
    Ok(transcript)
}

pub(super) fn conversation(
    view: &SheprdWindow,
    record: &Record,
    index: usize,
    bounds: Bounds,
) -> Result<Excerpt, UnavailableReason> {
    let transcript = transcript(view, record, index)?;
    let mut result = conversation_text(transcript, bounds);
    result.truncated |= transcript.truncated;
    Ok(result)
}

pub(super) fn conversation_text(
    transcript: &sheprd_chat_protocol::Transcript,
    bounds: Bounds,
) -> Excerpt {
    use sheprd_chat_protocol::EntryKind;
    let entries = transcript.entries.iter().filter_map(|entry| {
        let role = match &entry.kind {
            EntryKind::User { .. } => "User",
            EntryKind::Agent { .. } => "Agent",
            EntryKind::Thought { .. } => "Thought",
            EntryKind::Notice { .. } => "Notice",
            _ => return None,
        };
        let mut text = format!("{role}:\n");
        if let Some(value) = entry.kind.text() {
            push(&mut text, value, bounds.max_bytes + 4);
        }
        Some(text)
    });
    excerpt(entries, bounds)
}

pub(super) fn terminal(
    state: &LiveState,
    snapshot: &ClientShellSnapshot,
    pane: &str,
    bounds: Bounds,
) -> Result<Excerpt, UnavailableReason> {
    let surface = state
        .surface
        .as_ref()
        .ok_or(UnavailableReason::NotObserved)?;
    if surface.boot_id != snapshot.boot_id || surface.projection_revision != snapshot.revision {
        return Err(UnavailableReason::StaleTarget);
    }
    let rect = surface
        .panes
        .iter()
        .find(|candidate| candidate.pane_id == pane)
        .ok_or(UnavailableReason::NotObserved)?
        .inner_rect;
    let frame = &surface.frame;
    let width = usize::from(frame.width);
    let height = usize::from(frame.height);
    if usize::from(rect.x) + usize::from(rect.width) > width
        || usize::from(rect.y) + usize::from(rect.height) > height
        || frame.cells.len() < width * height
    {
        return Err(UnavailableReason::NotObserved);
    }
    let lines = (usize::from(rect.y)..usize::from(rect.y) + usize::from(rect.height)).map(|y| {
        let mut line = String::new();
        let mut clipped = false;
        for x in usize::from(rect.x)..usize::from(rect.x) + usize::from(rect.width) {
            let cell = &frame.cells[y * width + x];
            if !cell.skip && !push(&mut line, &cell.symbol, bounds.max_bytes + 4) {
                clipped = true;
                break;
            }
        }
        if clipped {
            line
        } else {
            line.trim_end().to_owned()
        }
    });
    Ok(excerpt(lines, bounds))
}
