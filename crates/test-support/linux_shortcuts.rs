// ABOUTME: Shares fixed native Linux cases and correlated fixture notices.
// ABOUTME: Keeps input sequences in compiled test data rather than terminal or log text.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    // keep-sorted start
    DecreaseFont,
    Fullscreen(bool),
    IncreaseFont,
    Keybindings,
    NewTab,
    NextRegion,
    NextTab,
    Palette,
    PolishSearch,
    PreviousRegion,
    PreviousTab,
    ResetFont,
    ResumePalette,
    Settings,
    SplitEditor,
    Tab(u8),
    TerminalBytes(&'static [u8]),
    // keep-sorted end
}

#[derive(Debug)]
pub(super) struct Case {
    pub(super) chords: &'static [&'static [u32]],
    pub(super) id: &'static str,
    pub(super) outcome: Outcome,
}

pub(super) const CAPTURE_FILE: &str = "linux-key-capture.sh";
pub(super) const PREFIX: &[u32] = &[29, 48];

#[path = "linux_shortcuts/capture.rs"]
pub(super) mod capture;

// Linux evdev codes: Ctrl=29, Shift=42, T=20, digits 1..9=2..10, AltGr=100.
pub(super) const CASES: &[Case] = &[
    Case {
        chords: &[&[29, 42, 20]],
        id: "new-tab",
        outcome: Outcome::NewTab,
    },
    Case {
        chords: &[&[29, 42, 2]],
        id: "tab-1",
        outcome: Outcome::Tab(1),
    },
    Case {
        chords: &[&[29, 42, 3]],
        id: "tab-2",
        outcome: Outcome::Tab(2),
    },
    Case {
        chords: &[&[29, 42, 4]],
        id: "tab-3",
        outcome: Outcome::Tab(3),
    },
    Case {
        chords: &[&[29, 42, 5]],
        id: "tab-4",
        outcome: Outcome::Tab(4),
    },
    Case {
        chords: &[&[29, 42, 6]],
        id: "tab-5",
        outcome: Outcome::Tab(5),
    },
    Case {
        chords: &[&[29, 42, 7]],
        id: "tab-6",
        outcome: Outcome::Tab(6),
    },
    Case {
        chords: &[&[29, 42, 8]],
        id: "tab-7",
        outcome: Outcome::Tab(7),
    },
    Case {
        chords: &[&[29, 42, 9]],
        id: "tab-8",
        outcome: Outcome::Tab(8),
    },
    Case {
        chords: &[&[29, 42, 10]],
        id: "tab-9",
        outcome: Outcome::Tab(9),
    },
    Case {
        chords: &[&[29, 42, 13]],
        id: "font-increase",
        outcome: Outcome::IncreaseFont,
    },
    Case {
        chords: &[&[29, 42, 12]],
        id: "font-decrease",
        outcome: Outcome::DecreaseFont,
    },
    Case {
        chords: &[&[29, 42, 11]],
        id: "font-reset",
        outcome: Outcome::ResetFont,
    },
    Case {
        chords: &[&[64]],
        id: "pane-f6",
        outcome: Outcome::TerminalBytes(b"\x1b[17~"),
    },
    Case {
        chords: &[&[42, 64]],
        id: "pane-shift-f6",
        outcome: Outcome::TerminalBytes(b"\x1b[17;2~"),
    },
    Case {
        chords: &[&[87]],
        id: "pane-f11",
        outcome: Outcome::TerminalBytes(b"\x1b[23~"),
    },
    Case {
        chords: &[&[29, 104]],
        id: "pane-ctrl-page-up",
        outcome: Outcome::TerminalBytes(b"\x1b[5;5~"),
    },
    Case {
        chords: &[&[29, 109]],
        id: "pane-ctrl-page-down",
        outcome: Outcome::TerminalBytes(b"\x1b[6;5~"),
    },
    Case {
        chords: &[&[29, 42, 109]],
        id: "direct-next-tab",
        outcome: Outcome::NextTab,
    },
    Case {
        chords: &[&[29, 42, 104]],
        id: "direct-previous-tab",
        outcome: Outcome::PreviousTab,
    },
    Case {
        chords: &[PREFIX, &[49]],
        id: "prefix-next-tab",
        outcome: Outcome::NextTab,
    },
    Case {
        chords: &[PREFIX, &[25]],
        id: "prefix-previous-tab",
        outcome: Outcome::PreviousTab,
    },
    Case {
        chords: &[&[29, 64]],
        id: "direct-next-region",
        outcome: Outcome::NextRegion,
    },
    Case {
        chords: &[&[29, 42, 64]],
        id: "direct-previous-region",
        outcome: Outcome::PreviousRegion,
    },
    Case {
        chords: &[PREFIX, &[64]],
        id: "prefix-next-region",
        outcome: Outcome::NextRegion,
    },
    Case {
        chords: &[PREFIX, &[42, 64]],
        id: "prefix-previous-region",
        outcome: Outcome::PreviousRegion,
    },
    Case {
        chords: &[&[29, 42, 87]],
        id: "direct-fullscreen-enter",
        outcome: Outcome::Fullscreen(true),
    },
    Case {
        chords: &[&[29, 42, 87]],
        id: "direct-fullscreen-leave",
        outcome: Outcome::Fullscreen(false),
    },
    Case {
        chords: &[PREFIX, &[87]],
        id: "prefix-fullscreen-enter",
        outcome: Outcome::Fullscreen(true),
    },
    Case {
        chords: &[PREFIX, &[87]],
        id: "prefix-fullscreen-leave",
        outcome: Outcome::Fullscreen(false),
    },
    Case {
        chords: &[&[29, 42, 53]],
        id: "keybindings",
        outcome: Outcome::Keybindings,
    },
    Case {
        chords: &[&[29, 42, 25]],
        id: "palette",
        outcome: Outcome::Palette,
    },
    Case {
        chords: &[&[29, 42, 19]],
        id: "palette-resume",
        outcome: Outcome::ResumePalette,
    },
    Case {
        chords: &[&[29, 42, 51]],
        id: "settings-alias",
        outcome: Outcome::Settings,
    },
    Case {
        chords: &[PREFIX, &[31]],
        id: "prefix-settings",
        outcome: Outcome::Settings,
    },
    Case {
        chords: &[&[29, 42, 43]],
        id: "split-editor-alias",
        outcome: Outcome::SplitEditor,
    },
    Case {
        chords: &[&[100, 38], &[42, 100, 49]],
        id: "polish-search",
        outcome: Outcome::PolishSearch,
    },
];

/// Mirrors the command catalog's one-based position within the focused workspace.
pub(super) fn tab_at_position(tabs: &[String], number: u8) -> Option<&str> {
    tabs.get(usize::from(number).checked_sub(1)?)
        .map(String::as_str)
}

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
