//! The words for [`sheprd_ago::Elapsed`]: "just now", "5 min ago",
//! "yesterday", "3 days ago". The crate below the app names the unit and its
//! count, and the app writes them, as every crate below the app leaves its
//! messages to the app (docs/i18n.md).
// i18n: migrated

use crate::i18n::{self, FluentValue};
use sheprd_ago::Elapsed;

/// The message and its count for `elapsed`.
fn message(elapsed: Elapsed) -> (&'static str, Option<u64>) {
    match elapsed {
        Elapsed::Now => ("ago-now", None),
        Elapsed::Minutes(count) => ("ago-minutes", Some(count)),
        Elapsed::Hours(count) => ("ago-hours", Some(count)),
        Elapsed::Days(count) => ("ago-days", Some(count)),
    }
}

fn args(count: Option<u64>) -> Vec<(&'static str, FluentValue<'static>)> {
    count
        .map(|count| ("count", FluentValue::from(count)))
        .into_iter()
        .collect()
}

/// `seconds` ago in the current locale.
pub(crate) fn text(seconds: u64) -> String {
    let (id, count) = message(sheprd_ago::elapsed(seconds));
    i18n::message(id, &args(count))
}

#[cfg(test)]
mod tests;
