//! The environment a chat's tab is created with: the `[chat.env]` pairs as the
//! Settings field shows and reads them, checked against what a `tab.create`
//! request can carry and what the app sets itself.
use crate::chat::PROGRAM_ENV;
use std::collections::BTreeMap;

/// Variables kept, and the bytes their names and values take together, so
/// a `tab.create` request stays small.
pub(crate) const MAX_ENV: usize = 64;
pub(crate) const MAX_ENV_BYTES: usize = 16 * 1024;

/// Names the app sets itself, which the table may not replace.
const RESERVED: [&str; 1] = [PROGRAM_ENV];

/// Why a `[chat.env]` table cannot become a tab's environment.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("`{name}` cannot be a chat environment variable")]
    Name { name: String },
    #[error(
        "[chat.env] holds more than {} variables or {} bytes",
        super::MAX_ENV,
        super::MAX_ENV_BYTES
    )]
    Size,
}

/// Every name a valid variable name and not one the app sets, every value
/// free of NUL, and the whole within [`MAX_ENV`] and [`MAX_ENV_BYTES`].
pub(crate) fn check_env(env: &BTreeMap<String, String>) -> Result<(), Error> {
    if env.len() > MAX_ENV {
        return Err(Error::Size);
    }
    let bytes: usize = env
        .iter()
        .map(|(name, value)| name.len() + value.len())
        .sum();
    if bytes > MAX_ENV_BYTES {
        return Err(Error::Size);
    }
    for (name, value) in env {
        let valid = name
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        if !valid || RESERVED.contains(&name.as_str()) || value.contains('\0') {
            return Err(Error::Name { name: name.clone() });
        }
    }
    Ok(())
}

/// The table as the Settings field shows it: `NAME=value` pairs separated
/// by spaces, a value with spaces or quotes in double quotes.
pub(crate) fn env_text(env: &BTreeMap<String, String>) -> String {
    env.iter()
        .map(|(name, value)| {
            if value.is_empty() || value.contains([' ', '"', '\\', '\t']) {
                let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
                format!("{name}=\"{escaped}\"")
            } else {
                format!("{name}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The pairs the Settings field holds, or `None` when one is malformed.
pub(crate) fn parse_env(text: &str) -> Option<BTreeMap<String, String>> {
    let mut env = BTreeMap::new();
    let mut chars = text.chars().peekable();
    loop {
        while chars.next_if(|ch| ch.is_whitespace()).is_some() {}
        if chars.peek().is_none() {
            break;
        }
        let mut name = String::new();
        while let Some(ch) = chars.next_if(|ch| *ch != '=' && !ch.is_whitespace()) {
            name.push(ch);
        }
        chars.next_if_eq(&'=')?;
        let mut value = String::new();
        if chars.next_if_eq(&'"').is_some() {
            loop {
                match chars.next()? {
                    '"' => break,
                    '\\' => value.push(chars.next()?),
                    ch => value.push(ch),
                }
            }
            if chars.peek().is_some_and(|ch| !ch.is_whitespace()) {
                return None;
            }
        } else {
            while let Some(ch) = chars.next_if(|ch| !ch.is_whitespace()) {
                value.push(ch);
            }
        }
        env.insert(name, value);
    }
    check_env(&env).ok().map(|()| env)
}
