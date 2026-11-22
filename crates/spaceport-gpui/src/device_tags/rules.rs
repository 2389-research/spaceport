//! Local device tag records and normalization.
//! Saved JSON and input bounds belong to the desktop preference owner.

use serde::{Deserialize, Serialize};

const MAX_TAGS: usize = 8;
const MAX_TAG_CHARS: usize = 24;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct ComputerTags {
    pub(super) tags: Vec<String>,
    pub(super) updated_ms: u64,
}

impl ComputerTags {
    pub(super) fn new<T: AsRef<str>>(tags: impl IntoIterator<Item = T>, updated_ms: u64) -> Self {
        let mut kept: Vec<String> = Vec::new();
        for tag in tags {
            let Some(tag) = normalized(tag.as_ref()) else {
                continue;
            };
            if kept.len() < MAX_TAGS && !kept.iter().any(|other| same(other, &tag)) {
                kept.push(tag);
            }
        }
        Self {
            tags: kept,
            updated_ms,
        }
    }
}

pub(crate) fn normalized(text: &str) -> Option<String> {
    let words = text
        .split_whitespace()
        .filter(|word| !word.chars().any(char::is_control))
        .collect::<Vec<_>>()
        .join(" ");
    let tag: String = words.chars().take(MAX_TAG_CHARS).collect();
    let tag = tag.trim_end();
    (!tag.is_empty()).then(|| tag.to_owned())
}

pub(crate) fn same(one: &str, other: &str) -> bool {
    one.to_lowercase() == other.to_lowercase()
}
