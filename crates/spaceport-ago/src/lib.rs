//! How long ago something happened, in the largest whole unit: "just now",
//! "5 min ago", "yesterday", "3 days ago" from seconds elapsed.
//!
//! A crate below the app has no messages (docs/i18n.md), so this names the
//! unit and its count, and the app writes the words.

/// The largest whole unit `seconds` fills, with how many of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elapsed {
    /// Under a minute.
    Now,
    /// Whole minutes, under an hour.
    Minutes(u64),
    /// Whole hours, under a day.
    Hours(u64),
    /// Whole days.
    Days(u64),
}

/// `seconds` elapsed, in the largest whole unit. A clock that moved back
/// reads as now rather than in the future, so callers saturate.
pub fn elapsed(seconds: u64) -> Elapsed {
    match seconds {
        0..60 => Elapsed::Now,
        60..3_600 => Elapsed::Minutes(seconds / 60),
        3_600..86_400 => Elapsed::Hours(seconds / 3_600),
        _ => Elapsed::Days(seconds / 86_400),
    }
}

#[cfg(test)]
mod tests;
