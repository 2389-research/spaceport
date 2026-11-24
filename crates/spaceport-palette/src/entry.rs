//! One row the palette lists, with the fields ranking reads and the ones the
//! app paints.
//!
//! Nothing here says a word a person reads: the app builds every label, badge
//! and detail from its own catalog and its own daemon state.

use crate::{action::Action, frecency::Key, search::Fields};
use spaceport_client::protocol::AgentStatus;

#[derive(Clone)]
pub struct Entry {
    pub label: String,
    pub detail: String,
    pub badge: String,
    pub action: Action,
    /// Index of the earlier row this one nests under, indented only while
    /// that row is visible so a search never leaves it hanging beneath nothing.
    pub parent: Option<usize>,
    /// The daemon-reported status of a Go To row's agent, if it has one.
    pub status: Option<AgentStatus>,
    /// How choosing this row is remembered, for rows ranked by frecency.
    pub frecency: Option<Key>,
    /// Ranking points from how often and recently the row was chosen.
    pub boost: u32,
    pub(crate) fields: Fields,
}

impl Entry {
    pub fn new(
        label: String,
        detail: String,
        badge: impl Into<String>,
        action: Action,
        parent: Option<usize>,
    ) -> Self {
        Self::with_keywords(label, detail, badge, action, parent, "")
    }

    /// `keywords` are searchable but never shown, such as an agent's kind.
    pub fn with_keywords(
        label: String,
        detail: String,
        badge: impl Into<String>,
        action: Action,
        parent: Option<usize>,
        keywords: &str,
    ) -> Self {
        let badge = badge.into();
        let id = match &action {
            Action::Configured(id, _) => id.as_str(),
            _ => "",
        };
        // The detail leads the context so match indices map back onto it.
        let fields = Fields::new(&label, &format!("{detail} {badge} {id} {keywords}"));
        Self {
            label,
            detail,
            badge,
            action,
            parent,
            status: None,
            frecency: None,
            boost: 0,
            fields,
        }
    }
}
