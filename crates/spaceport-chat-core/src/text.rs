//! What a chat measures: a duration in its largest whole unit, and the turn
//! summary a measured entry shows. A crate below the app has no messages
//! (`docs/i18n.md`), so this names the unit and its count, and the app writes
//! the words.

use spaceport_chat_protocol::{Tokens, TurnEnd, TurnSummary};

/// A duration in the largest whole unit it fills, with the parts the app
/// writes beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elapsed {
    /// Under a second.
    Milliseconds(u64),
    /// Whole seconds, under a minute.
    Seconds(u64),
    /// Whole minutes under an hour, and the seconds left over.
    Minutes { minutes: u64, seconds: u64 },
    /// Whole hours, and the minutes left over.
    Hours { hours: u64, minutes: u64 },
}

/// `ms` in the largest whole unit it fills.
pub fn duration(ms: u64) -> Elapsed {
    let seconds = ms / 1000;
    match seconds {
        0 => Elapsed::Milliseconds(ms),
        1..60 => Elapsed::Seconds(seconds),
        60..3600 => Elapsed::Minutes {
            minutes: seconds / 60,
            seconds: seconds % 60,
        },
        _ => Elapsed::Hours {
            hours: seconds / 3600,
            minutes: seconds % 3600 / 60,
        },
    }
}

/// A turn as the app shows it: how it ended, how long it took, and the tokens
/// it used when the agent reports them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measured {
    pub elapsed: Elapsed,
    pub end: TurnEnd,
    pub tokens: Option<Tokens>,
}

/// What `summary` measured.
pub fn turn(summary: &TurnSummary) -> Measured {
    Measured {
        elapsed: duration(summary.duration_ms),
        end: summary.end,
        tokens: summary.tokens,
    }
}

#[cfg(test)]
mod tests;
