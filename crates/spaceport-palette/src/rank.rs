//! How a filter's rows are ordered.

use crate::{
    agent_filter::AgentFilter,
    entry::Entry,
    filter::Filter,
    search::{Hit, Query},
};

/// Ranks the `filter`'s rows. The flat project list has no order of its own
/// to keep, so with no query its most frecent projects come first.
pub fn rank_rows(
    filter: Filter,
    status: AgentFilter,
    entries: &[Entry],
    candidates: Vec<usize>,
    query: &Query,
    matcher: &mut nucleo_matcher::Matcher,
) -> Vec<Hit> {
    let mut hits = status.rank(entries, candidates, query, matcher);
    if filter == Filter::Projects && query.is_empty() {
        hits.sort_by_key(|hit| std::cmp::Reverse(entries[hit.index].boost));
    }
    hits
}
