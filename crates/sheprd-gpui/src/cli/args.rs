//! ABOUTME: Parses control options separately from literal command text.
//! ABOUTME: Keeps OS strings intact until a command requires UTF-8.
use super::CliError;
use std::{collections::VecDeque, ffi::OsString};

pub(crate) struct Args {
    rest: VecDeque<OsString>,
    /// Free text or everything after `--`; these words cannot be options.
    trailing: VecDeque<OsString>,
    /// The command being parsed, named in errors.
    command: &'static str,
    /// The window to act in: the one `--window` names, or the one a
    /// command takes as its own argument, which wins.
    window: Option<u64>,
}

fn is_flag(value: &OsString) -> bool {
    value.as_encoded_bytes().starts_with(b"-") && value.len() > 1
}

/// Find literal text before any eager option or help scan can consume it.
fn text_boundary(args: &VecDeque<OsString>, command: &str) -> Option<(usize, bool)> {
    let mut positionals = 0;
    let mut text_after = None;
    let mut at = 0;
    while let Some(arg) = args.get(at) {
        match arg.to_str() {
            Some("--") => return Some((at, true)),
            Some("--window") => {
                at += 1;
                if args.get(at).is_some_and(|value| !is_flag(value)) {
                    at += 1;
                }
            }
            Some("--json" | "--help" | "-h") => at += 1,
            _ if text_after == Some(positionals) => return Some((at, false)),
            _ => {
                if !is_flag(arg) {
                    positionals += 1;
                    if positionals == 1 && arg == "set" {
                        text_after = match command {
                            "notes" | "theme" | "font" => Some(1),
                            "config" => Some(2),
                            _ => None,
                        };
                    }
                }
                at += 1;
            }
        }
    }
    None
}

impl Args {
    pub(crate) fn new(args: impl IntoIterator<Item = OsString>, command: &'static str) -> Self {
        let mut rest: VecDeque<OsString> = args.into_iter().collect();
        let trailing = text_boundary(&rest, command)
            .map(|(at, separator)| {
                let mut text = rest.split_off(at);
                if separator {
                    let _ = text.pop_front();
                }
                text
            })
            .unwrap_or_default();
        Self {
            rest,
            trailing,
            command,
            window: None,
        }
    }

    pub(crate) fn window(&self) -> Option<u64> {
        self.window
    }

    pub(crate) fn set_window(&mut self, window: Option<u64>) {
        self.window = window;
    }

    pub(crate) fn name(&self) -> &'static str {
        self.command
    }

    /// The next word, which names a subcommand.
    pub(crate) fn verb(&mut self) -> Option<OsString> {
        self.rest.pop_front().or_else(|| self.trailing.pop_front())
    }

    /// Help wins over other options, but never consumes literal text.
    pub(crate) fn wants_help(&self) -> bool {
        self.rest.iter().any(|arg| arg == "--help" || arg == "-h")
    }

    /// Removes `--flag` before literal text; whether it was there.
    pub(crate) fn flag(&mut self, flag: &str) -> bool {
        let before = self.rest.len();
        self.rest.retain(|arg| arg != flag);
        self.rest.len() != before
    }

    /// Removes `--flag VALUE` before literal text. Its value cannot be a flag.
    pub(crate) fn option(&mut self, flag: &'static str) -> Result<Option<OsString>, CliError> {
        let Some(at) = self.rest.iter().position(|arg| arg == flag) else {
            return Ok(None);
        };
        let _ = self.rest.remove(at);
        let value = self
            .rest
            .remove(at)
            .filter(|value| !value.is_empty() && !is_flag(value))
            .ok_or(CliError::MissingValue(flag))?;
        if self.rest.iter().any(|arg| arg == flag) {
            return Err(CliError::DuplicateOption(flag));
        }
        Ok(Some(value))
    }

    /// Every `--flag VALUE`, for a flag that may repeat.
    pub(crate) fn options(&mut self, flag: &'static str) -> Result<Vec<OsString>, CliError> {
        let mut values = Vec::new();
        while let Some(at) = self.rest.iter().position(|arg| arg == flag) {
            let _ = self.rest.remove(at);
            values.push(
                self.rest
                    .remove(at)
                    .filter(|value| !value.is_empty() && !is_flag(value))
                    .ok_or(CliError::MissingValue(flag))?,
            );
        }
        Ok(values)
    }

    pub(crate) fn text_option(&mut self, flag: &'static str) -> Result<Option<String>, CliError> {
        self.option(flag)?.map(utf8).transpose()
    }

    /// `--flag N`, a whole number within `range`.
    pub(crate) fn number<T>(
        &mut self,
        flag: &'static str,
        range: std::ops::RangeInclusive<T>,
    ) -> Result<Option<T>, CliError>
    where
        T: std::str::FromStr + PartialOrd,
    {
        let Some(value) = self.option(flag)? else {
            return Ok(None);
        };
        value
            .to_str()
            .and_then(|text| text.parse::<T>().ok())
            .filter(|number| range.contains(number))
            .map(Some)
            .ok_or(CliError::InvalidValue(flag))
    }

    /// The next positional argument. Literal tails may contain flag words.
    pub(crate) fn positional(&mut self) -> Option<OsString> {
        self.rest
            .iter()
            .position(|arg| !is_flag(arg))
            .and_then(|at| self.rest.remove(at))
            .or_else(|| self.trailing.pop_front())
    }

    pub(crate) fn required(&mut self, what: &'static str) -> Result<OsString, CliError> {
        self.positional().ok_or(CliError::MissingArgument {
            command: self.command,
            what,
        })
    }

    pub(crate) fn required_text(&mut self, what: &'static str) -> Result<String, CliError> {
        utf8(self.required(what)?)
    }

    pub(crate) fn optional_text(&mut self) -> Result<Option<String>, CliError> {
        self.positional().map(utf8).transpose()
    }

    /// Every remaining positional argument joined by spaces, as a prompt or
    /// a note is typed unquoted.
    pub(crate) fn words(&mut self) -> Result<Option<String>, CliError> {
        let words = self
            .rest
            .drain(..)
            .chain(self.trailing.drain(..))
            .map(utf8)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((!words.is_empty()).then(|| words.join(" ")))
    }

    /// Fails on anything left over, so a typo is never silently ignored.
    pub(crate) fn finish(&mut self) -> Result<(), CliError> {
        match self.rest.pop_front().or_else(|| self.trailing.pop_front()) {
            None => Ok(()),
            Some(extra) => Err(CliError::Unexpected {
                command: self.command,
                argument: extra,
            }),
        }
    }
}

pub(crate) fn utf8(value: OsString) -> Result<String, CliError> {
    value.into_string().map_err(CliError::InvalidEncoding)
}

#[cfg(test)]
mod tests;
