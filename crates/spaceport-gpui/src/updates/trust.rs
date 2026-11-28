// ABOUTME: Reads the compiled Sparkle trust inputs every signed channel checks.
// ABOUTME: Files each channel's appcast URL under the channel's own name.
use super::Error;
use crate::constants::Channel;
use std::collections::BTreeMap;

/// The public trust inputs compiled in from `scripts/release/sparkle.json`, the
/// file packaging, feed creation and publication verify too.
pub(super) struct Trust {
    feeds: BTreeMap<String, String>,
    pub(super) public_key: String,
}

#[derive(serde::Deserialize)]
struct RawTrust {
    feeds: BTreeMap<String, RawFeed>,
    public_key: String,
}

/// One signed channel's feed. The release the asset lives on is packaging's
/// and publication's business; this side only needs the address to check.
#[derive(serde::Deserialize)]
struct RawFeed {
    url: String,
}

impl Trust {
    /// This build's compiled trust inputs.
    pub(super) fn compiled() -> Result<Self, Error> {
        Self::parse(include_str!("../../../../scripts/release/sparkle.json"))
    }

    pub(super) fn parse(json: &str) -> Result<Self, Error> {
        let raw: RawTrust = serde_json::from_str(json)?;
        let mut feeds = BTreeMap::new();
        for (channel, feed) in raw.feeds {
            // A feed the app downloads from is an https address or it is not a feed.
            if !is_https_url(&feed.url) {
                return Err(Error::InsecureFeed(feed.url));
            }
            feeds.insert(channel, feed.url);
        }
        if raw.public_key.trim().is_empty() {
            return Err(Error::MissingPublicKey);
        }
        Ok(Self {
            feeds,
            public_key: raw.public_key,
        })
    }

    /// The signed appcast a build on `channel` must carry, or `None` for a
    /// channel that never updates.
    pub(super) fn feed(&self, channel: Channel) -> Option<&str> {
        self.feeds.get(channel.name()).map(String::as_str)
    }
}

/// Whether `url` is an absolute https address with a host and a path.
fn is_https_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    !authority.is_empty()
        && !authority.contains('@')
        && !authority.contains(char::is_whitespace)
        && !path.is_empty()
}

#[cfg(test)]
#[path = "tests/trust.rs"]
mod tests;
