//! What a chat shows about a turn: a duration in its largest whole unit
//! (`950ms`, `12s`, `3m 4s`, `1h 2m`), and how the turn ended with how long it
//! took and the tokens it used (`Worked 12s · 3.2k in · 450 out`).

use spaceport_chat_protocol::{TurnEnd, TurnSummary, display};
use spaceport_i18n::fl;

/// A duration in the largest whole unit it fills, with the parts written
/// beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Elapsed {
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
fn elapsed(ms: u64) -> Elapsed {
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

/// The words for a measured duration.
fn words(elapsed: Elapsed) -> String {
    match elapsed {
        Elapsed::Milliseconds(ms) => fl!("chat-text-milliseconds", ms = ms),
        Elapsed::Seconds(seconds) => fl!("chat-text-seconds", seconds = seconds),
        Elapsed::Minutes { minutes, seconds } => {
            fl!("chat-text-minutes", minutes = minutes, seconds = seconds)
        }
        Elapsed::Hours { hours, minutes } => {
            fl!("chat-text-hours", hours = hours, minutes = minutes)
        }
    }
}

/// `950ms`, `12s`, `3m 4s`, or `1h 2m`.
pub fn duration(ms: u64) -> String {
    words(elapsed(ms))
}

/// `Worked 12s · 3.2k in · 450 out`: how a turn ended, how long it took, and
/// the tokens it used when the agent reports them.
pub fn turn(summary: &TurnSummary) -> String {
    let duration = duration(summary.duration_ms);
    let ended = match summary.end {
        TurnEnd::Done => fl!("chat-text-turn-done", duration = duration),
        TurnEnd::Cancelled => fl!("chat-text-turn-cancelled", duration = duration),
        TurnEnd::Limit => fl!("chat-text-turn-limit", duration = duration),
        TurnEnd::Refused => fl!("chat-text-turn-refused", duration = duration),
        TurnEnd::Failed => fl!("chat-text-turn-failed", duration = duration),
        TurnEnd::Other => fl!("chat-text-turn-other", duration = duration),
    };
    let Some(tokens) = summary.tokens else {
        return ended;
    };
    // Token counts in the protocol's short form: `950`, `18.6k`, `1.2M`.
    let (input, output) = (
        display::tokens(tokens.input),
        display::tokens(tokens.output),
    );
    if tokens.thought > 0 {
        fl!(
            "chat-text-turn-tokens-thinking",
            turn = ended,
            input = input,
            output = output,
            thought = display::tokens(tokens.thought)
        )
    } else {
        fl!(
            "chat-text-turn-tokens",
            turn = ended,
            input = input,
            output = output
        )
    }
}

#[cfg(test)]
mod tests;
