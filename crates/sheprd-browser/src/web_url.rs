//! A bounded http or https address with a host and no whitespace or controls.
//! Opening one is the app's, through the system handler; nothing here loads a
//! page.
use crate::{Error, Result};

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
            return Err(Error::InvalidBrowserUrl);
        }
        let url = url::Url::parse(value).map_err(|_| Error::InvalidBrowserUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none_or(str::is_empty) {
            return Err(Error::InvalidBrowserUrl);
        }
        // Serialization can lengthen an address, for example by
        // percent-encoding, so the bound applies to what a page is given too.
        if url.as_str().len() > MAX_URL_BYTES {
            return Err(Error::InvalidBrowserUrl);
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn only_bounded_web_addresses_with_a_host_are_accepted() {
        for valid in [
            "http://localhost:3000/",
            "https://example.com/a?b#c",
            "https://[::1]:8080/",
        ] {
            assert!(WebUrl::try_from(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,hi",
            "vscode://open",
            "https://",
            "https://a.test/\nb",
            "https://a.test/ b",
            "not a url",
        ] {
            assert!(
                matches!(WebUrl::try_from(invalid), Err(Error::InvalidBrowserUrl)),
                "{invalid}"
            );
        }
        let long = format!("https://a.test/{}", "x".repeat(MAX_URL_BYTES));
        assert!(WebUrl::try_from(long.as_str()).is_err());
    }

    #[test]
    fn saved_addresses_are_validated_when_read() {
        assert!(serde_json::from_str::<WebUrl>(r#""https://a.test/""#).is_ok());
        assert!(serde_json::from_str::<WebUrl>(r#""file:///etc/passwd""#).is_err());
    }
}
