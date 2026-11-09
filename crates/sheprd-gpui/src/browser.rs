//! Client-owned native tabs and shared terminal/chat split layouts.
//! The selected Herdr runtime owns terminal processes; views only arrange them.
// i18n: migrated

mod closed;
mod group_motion;
mod group_settle;
mod groups;
mod groups_view;
mod hidden;
mod layouts;
mod location;
mod store;
mod tab_appear;
pub(crate) mod tab_body;
mod tab_move;
mod tab_scroll;
mod view;
#[cfg(test)]
mod view_tests;

pub(crate) use closed::ClosedHerdrTab;
pub(crate) use group_motion::Fold;
#[cfg(test)]
pub(crate) use groups::GroupIds;
pub(crate) use groups::{GroupId, Pick, Shown, Slot};
pub(crate) use layouts::Layouts;
#[cfg(test)]
pub(crate) use location::LocalFile;
pub(crate) use location::Location;
#[cfg(test)]
pub(crate) use location::RetiredOrchestratorRepo;
pub(crate) use store::{CoreGroupKey, Scope, Store, Tab, TabId};
pub(crate) use tab_appear::{Leaving, Listed};
pub(crate) use tab_scroll::{Thumb, ThumbDrag};
pub(crate) use view::{Browser, PageOpened, scope};

const MAX_URL_BYTES: usize = 8192;

/// A bounded http or https address with a host and no whitespace or controls.
/// Explicit terminal and service link clicks use this before opening externally.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct WebUrl(url::Url);

impl WebUrl {
    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(crate) fn host(&self) -> &str {
        self.0.host_str().unwrap_or_default()
    }
}

impl TryFrom<&str> for WebUrl {
    type Error = crate::Error;

    fn try_from(value: &str) -> crate::Result<Self> {
        if value.len() > MAX_URL_BYTES || value.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(crate::Error::InvalidBrowserUrl);
        }
        let url = url::Url::parse(value).map_err(|_| crate::Error::InvalidBrowserUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none_or(str::is_empty) {
            return Err(crate::Error::InvalidBrowserUrl);
        }
        // Serialization can lengthen an address, for example by
        // percent-encoding, so the bound applies to what a page is given too.
        if url.as_str().len() > MAX_URL_BYTES {
            return Err(crate::Error::InvalidBrowserUrl);
        }
        Ok(Self(url))
    }
}

impl TryFrom<String> for WebUrl {
    type Error = crate::Error;

    fn try_from(value: String) -> crate::Result<Self> {
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
                matches!(
                    WebUrl::try_from(invalid),
                    Err(crate::Error::InvalidBrowserUrl)
                ),
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
