// ABOUTME: Shares fixed native Linux cases and correlated fixture notices.
// ABOUTME: Keeps input sequences in compiled test data rather than terminal or log text.

#[derive(Clone, Copy, Debug)]
pub(super) enum Outcome {
    // keep-sorted start
    NewTab,
    PolishSearch,
    // keep-sorted end
}

#[derive(Debug)]
pub(super) struct Case {
    pub(super) chords: &'static [&'static [u32]],
    pub(super) id: &'static str,
    pub(super) outcome: Outcome,
}

// Linux evdev codes: Left Ctrl, Left Shift, T; Right Alt, L, and N.
pub(super) const CASES: &[Case] = &[
    Case {
        chords: &[&[29, 42, 20]],
        id: "new-tab",
        outcome: Outcome::NewTab,
    },
    Case {
        chords: &[&[100, 38], &[42, 100, 49]],
        id: "polish-search",
        outcome: Outcome::PolishSearch,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Notice<'a> {
    // keep-sorted start
    Complete,
    Mapped,
    Passed(&'a str),
    Ready(&'a str),
    // keep-sorted end
}

pub(super) fn valid_nonce(nonce: &str) -> bool {
    (16..=64).contains(&nonce.len()) && nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn notice(nonce: &str, value: Notice<'_>) -> String {
    let suffix = match value {
        Notice::Mapped => "mapped".to_owned(),
        Notice::Ready(id) => format!("ready {id}"),
        Notice::Passed(id) => format!("passed {id}"),
        Notice::Complete => "complete".to_owned(),
    };
    format!("LINUX_KEYS {nonce} {suffix}")
}

#[cfg(test)]
pub(super) fn parse_notice<'a>(line: &'a str, nonce: &str) -> Option<Notice<'a>> {
    if !valid_nonce(nonce) {
        return None;
    }
    let (actual, suffix) = line.strip_prefix("LINUX_KEYS ")?.split_once(' ')?;
    if actual != nonce {
        return None;
    }
    match suffix {
        "mapped" => Some(Notice::Mapped),
        "complete" => Some(Notice::Complete),
        _ => {
            let (kind, id) = suffix.split_once(' ')?;
            if !CASES.iter().any(|case| case.id == id) {
                return None;
            }
            match kind {
                "ready" => Some(Notice::Ready(id)),
                "passed" => Some(Notice::Passed(id)),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
#[path = "linux_shortcuts/tests.rs"]
mod tests;
