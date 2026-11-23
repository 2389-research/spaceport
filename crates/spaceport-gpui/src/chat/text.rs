//! The words for what `spaceport-chat-core` measures: `950ms`, `12s`,
//! `3m 4s`, `1h 2m`, and `Worked 12s · 3.2k in · 450 out`. The crate measures
//! and the app writes the words, as every crate below the app leaves its
//! messages to the app (`docs/i18n.md`).

use crate::i18n::fl;
use spaceport_chat_core::text::Elapsed;
use spaceport_chat_protocol::{TurnEnd, TurnSummary, display};

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
pub(crate) fn duration(ms: u64) -> String {
    words(spaceport_chat_core::text::duration(ms))
}

/// `Worked 12s · 3.2k in · 450 out`: how a turn ended, how long it took,
/// and the tokens it used when the agent reports them.
pub(crate) fn turn(summary: &TurnSummary) -> String {
    let measured = spaceport_chat_core::text::turn(summary);
    let duration = words(measured.elapsed);
    let ended = match measured.end {
        TurnEnd::Done => fl!("chat-text-turn-done", duration = duration),
        TurnEnd::Cancelled => fl!("chat-text-turn-cancelled", duration = duration),
        TurnEnd::Limit => fl!("chat-text-turn-limit", duration = duration),
        TurnEnd::Refused => fl!("chat-text-turn-refused", duration = duration),
        TurnEnd::Failed => fl!("chat-text-turn-failed", duration = duration),
        TurnEnd::Other => fl!("chat-text-turn-other", duration = duration),
    };
    let Some(tokens) = measured.tokens else {
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
