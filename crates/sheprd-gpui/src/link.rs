//! `spaceport://` links: what the shell hands the app when the user
//! picks a jump list entry, clicks a notification, or follows a link from a
//! browser or another app.
//!
//! A link is untrusted input. Any web page can link one, and any local
//! process can send one, so this is the only parser and it accepts exactly:
//!
//! - `spaceport://device?id=ID`: a device (host) to switch to;
//! - `spaceport://notification?tag=TAG`: the notification that was clicked.
//!
//! Nothing else: no other parts, no unknown or repeated parameters, bounded
//! lengths, and no control characters. A link names something to show; it
//! never carries a command, and the app never runs one because of it.

pub(crate) mod follow;
#[cfg(test)]
mod tests;

use url::Url;

pub(crate) const SCHEME: &str = "spaceport";
/// Bytes in a whole link. Browsers allow far more; nothing here needs it.
const MAX_LINK: usize = 4096;
const MAX_DEVICE: usize = 256;
const MAX_TAG: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    Device { id: String },
    Notification { tag: String },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum Error {
    #[error("the link is longer than {MAX_LINK} bytes")]
    TooLong,
    #[error("not a {SCHEME}:// link")]
    Scheme,
    #[error("the link is malformed")]
    Malformed(#[source] url::ParseError),
    #[error("the link has a part {SCHEME}:// links never have")]
    UnexpectedPart,
    #[error("unknown link action {0:?}")]
    UnknownAction(String),
    #[error("unknown link parameter {0:?}")]
    UnknownParameter(String),
    #[error("the link repeats {0}")]
    DuplicateParameter(&'static str),
    #[error("the link needs {0}")]
    MissingParameter(&'static str),
    #[error("the link's device is not a device ID")]
    Device,
    #[error("the link's notification is not one this app posts")]
    Tag,
}

/// Whether a command-line argument is a link rather than an option, so a
/// malformed one is reported as a link instead of an unknown option.
pub(crate) fn is_link(argument: &std::ffi::OsStr) -> bool {
    let bytes = argument.as_encoded_bytes();
    bytes.len() > SCHEME.len()
        && bytes[..SCHEME.len()].eq_ignore_ascii_case(SCHEME.as_bytes())
        && bytes[SCHEME.len()] == b':'
}

impl Link {
    pub(crate) fn parse(text: &str) -> Result<Self, Error> {
        if text.len() > MAX_LINK {
            return Err(Error::TooLong);
        }
        let url = Url::parse(text).map_err(Error::Malformed)?;
        if url.scheme() != SCHEME {
            return Err(Error::Scheme);
        }
        if !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
            || url.fragment().is_some()
            || !matches!(url.path(), "" | "/")
        {
            return Err(Error::UnexpectedPart);
        }
        let action = url.host_str().unwrap_or_default();
        if !matches!(action, "device" | "notification") {
            return Err(Error::UnknownAction(crate::notifications::safe_text(
                action, 64,
            )));
        }
        let mut parameters = Parameters::default();
        for (key, value) in url.query_pairs() {
            parameters.add(&key, value.into_owned())?;
        }
        match action {
            "device" => {
                parameters.only(&["id"])?;
                let id = parameters.take("id")?;
                if id.len() > MAX_DEVICE || !id.chars().all(|c| c.is_ascii_graphic()) {
                    return Err(Error::Device);
                }
                Ok(Self::Device { id })
            }
            "notification" => {
                parameters.only(&["tag"])?;
                let tag = parameters.take("tag")?;
                // `herdr:` for daemon notices, `herdr-preview:` for the QA one.
                if tag.len() > MAX_TAG
                    || !tag.starts_with("herdr")
                    || tag.chars().any(crate::notifications::unsafe_char)
                {
                    return Err(Error::Tag);
                }
                Ok(Self::Notification { tag })
            }
            other => Err(Error::UnknownAction(crate::notifications::safe_text(
                other, 64,
            ))),
        }
    }

    /// The link as text, which [`Link::parse`] reads back to the same link.
    pub(crate) fn to_url(&self) -> String {
        let (action, key, value) = match self {
            Self::Device { id } => ("device", "id", id.as_str()),
            Self::Notification { tag } => ("notification", "tag", tag.as_str()),
        };
        let query: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair(key, &value)
            .finish();
        format!("{SCHEME}://{action}?{query}")
    }
}
/// The query, each key at most once.
#[derive(Default)]
struct Parameters(Vec<(&'static str, String)>);

const KEYS: [&str; 2] = ["id", "tag"];

impl Parameters {
    fn add(&mut self, key: &str, value: String) -> Result<(), Error> {
        let Some(key) = KEYS.into_iter().find(|known| *known == key) else {
            return Err(Error::UnknownParameter(crate::notifications::safe_text(
                key, 64,
            )));
        };
        if self.0.iter().any(|(seen, _)| *seen == key) {
            return Err(Error::DuplicateParameter(key));
        }
        self.0.push((key, value));
        Ok(())
    }

    /// Refuses a key this action does not take.
    fn only(&self, allowed: &[&str]) -> Result<(), Error> {
        match self.0.iter().find(|(key, _)| !allowed.contains(key)) {
            Some((key, _)) => Err(Error::UnknownParameter((*key).to_owned())),
            None => Ok(()),
        }
    }

    fn take(&mut self, key: &'static str) -> Result<String, Error> {
        let at = self
            .0
            .iter()
            .position(|(seen, _)| *seen == key)
            .ok_or(Error::MissingParameter(key))?;
        Ok(self.0.swap_remove(at).1)
    }
}
