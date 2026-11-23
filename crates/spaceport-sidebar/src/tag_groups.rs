//! Devices grouped by tag: a tag's colour, and whether the switch over the list
//! and the filter it set still have anything to offer. Sections are folded by
//! the app, which keys its own folds and words every heading; these are the
//! rules it asks about them.

use crate::device_tags::{Tags, same};
use spaceport_settings::{Theme, mix};

/// The ANSI colors a tag takes, leaving out green, which says online.
const HUES: [usize; 4] = [3, 4, 5, 6];

/// A tag's color: from its letters, so it is the same on every machine.
pub fn tag_color(theme: &Theme, tag: &str) -> u32 {
    let sum: usize = tag.to_lowercase().bytes().map(usize::from).sum();
    let hue = theme.palette[HUES[sum % HUES.len()]];
    theme.ink(mix(theme.foreground, hue, 44))
}

/// Whether some device carries `tag`, whatever its case.
pub fn in_use(tags: &Tags, tag: &str) -> bool {
    tags.all().iter().any(|known| same(known, tag))
}

/// Whether the switch shows: with one tag, its sections already say it all.
pub fn switchable(tags: &[&str]) -> bool {
    tags.len() > 1
}

/// Whether `filter` can no longer be cleared, its switch or segment gone.
pub fn filter_stale(tags: &Tags, filter: &str) -> bool {
    !switchable(&tags.all()) || !in_use(tags, filter)
}

/// Whether two devices list together: `work` and `Work` are one section.
pub fn same_section(one: Option<&str>, other: Option<&str>) -> bool {
    match (one, other) {
        (Some(one), Some(other)) => same(one, other),
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests;
