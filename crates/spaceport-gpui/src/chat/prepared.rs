//! What the view draws for an entry, prepared when the entry changes rather
//! than on every frame: the model is [`spaceport_chat_core::prepared`]'s, and
//! this module keeps the two things only a window has — the pictures it draws,
//! and the selectable projection of the same text — and answers the model's
//! wants.

use super::{
    labels::Labels,
    selection::Document,
    visual::{VisualId, Visuals},
};
use spaceport_chat_protocol::{EntryId, Transcript};
use std::{collections::HashSet, rc::Rc};

// The prepared entry model at the paths the rest of the chat reads it at.
pub(crate) use spaceport_chat_core::prepared::{
    Body, DiffSummary, Part, ToolSummary, folded_thought, notice_label, shown_text, tool_header,
    tool_text,
};

/// Bodies by entry, the pictures those bodies draw, and the selectable text of
/// the same entries.
#[derive(Default)]
pub(crate) struct Prepared {
    core: spaceport_chat_core::prepared::Prepared,
    text: Document,
    pub(crate) visuals: Visuals,
}

impl Prepared {
    pub(crate) fn body(&self, id: EntryId) -> Option<Rc<Body>> {
        self.core.body(id)
    }

    /// Whether the message part at `key` is still the card `document`.
    pub(crate) fn shows_widget(
        &self,
        key: super::widgets::Key,
        document: &super::widgets::Document,
    ) -> bool {
        self.core.shows_widget(key, document)
    }

    pub(crate) fn changed(&self) -> &[DiffSummary] {
        self.core.changed()
    }

    /// The steps the sub-agent of the call `call` took, in order.
    pub(crate) fn steps(&self, call: &str) -> &[EntryId] {
        self.core.steps(call)
    }

    pub(crate) fn text(&self) -> &Document {
        &self.text
    }

    /// Remote links display their destination beside the label. Keep the
    /// cached selection projection in the same coordinate system as the view.
    pub(crate) fn set_remote_links(&mut self, remote: bool) {
        self.text.set_remote_links(remote);
    }

    /// Apply before `update`: folded records contribute only their visible
    /// label/gist, never undisclosed source text between selected messages.
    pub(crate) fn set_expanded(&mut self, expanded: &HashSet<EntryId>) {
        self.text.set_expanded(expanded);
    }

    /// The entries that draw `visual`, to measure again once it lands.
    pub(crate) fn drawing(&self, visual: VisualId) -> impl Iterator<Item = EntryId> + '_ {
        self.core.drawing(visual)
    }

    /// Prepares the entries whose revision moved and forgets those gone.
    pub(crate) fn update(
        &mut self,
        transcript: &Transcript,
        keys: &[(EntryId, u64)],
        labels: &Labels,
    ) {
        self.core
            .update(transcript, keys, labels, super::rich::destination);
        // Wants before the retain, not after: a want over the picture budget is
        // staged while the pass is still discovering pictures, and only the
        // retain that follows frees the slots it is re-offered in. Reversing
        // the two drops the pictures of an entry whose parts were replaced.
        for want in self.core.take_wants() {
            self.visuals.want(want.id(), want.into_source());
        }
        self.visuals.retain(&self.core.drawn());
        self.text
            .update(transcript, keys, self.core.bodies(), labels);
    }

    /// Drops the prepared bodies and their selectable text, keeping the
    /// pictures, so the next [`Self::update`] writes every label again in
    /// the language now shown.
    pub(crate) fn forget_text(&mut self) {
        self.core.forget_bodies();
        self.text.clear();
    }

    /// Starts over, as after a snapshot that may reuse entry ids.
    pub(crate) fn clear(&mut self) {
        self.core.clear();
        self.text.clear();
        self.visuals.clear();
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests;
