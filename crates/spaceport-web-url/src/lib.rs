//! A bounded http or https address with a host and no whitespace or controls.
//! Opening one is the app's, through the system handler; nothing here loads a
//! page.
mod error;
#[cfg(test)]
mod tests;

pub use error::{Error, Result};

const MAX_URL_BYTES: usize = 8192;

/// A bounded http or https address with a host and no whitespace or controls.
/// Explicit terminal and service link clicks use this before opening externally.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WebUrl(url::Url);

impl WebUrl {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn host(&self) -> &str {
        self.0.host_str().unwrap_or_default()
    }
}

impl TryFrom<&str> for WebUrl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        if value.len() > MAX_URL_BYTES || value.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(Error::InvalidUrl);
        }
        let url = url::Url::parse(value).map_err(|_| Error::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none_or(str::is_empty) {
            return Err(Error::InvalidUrl);
        }
        // Serialization can lengthen an address, for example by
        // percent-encoding, so the bound applies to what a page is given too.
        if url.as_str().len() > MAX_URL_BYTES {
            return Err(Error::InvalidUrl);
        }
        Ok(Self(url))
    }
}

impl TryFrom<String> for WebUrl {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

impl From<WebUrl> for String {
    fn from(url: WebUrl) -> Self {
        url.0.into()
    }
}
