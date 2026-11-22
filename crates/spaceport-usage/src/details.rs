// ABOUTME: Provider-specific report shapes retained by the native usage panel.
// ABOUTME: These data types describe observed details without reading accounts or credentials.
use crate::model::Window;
use serde::Deserialize;

/// Daily spend for the panel's chart, oldest first.
#[derive(Debug, Default, PartialEq)]
pub struct OpenaiDaily {
    pub days: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClaudeAccount {
    pub email: String,
    pub alias: Option<String>,
    /// Held out of cswap's rotation; still usable when chosen by name.
    pub disabled: bool,
    pub status: ClaudeAccountStatus,
    /// The windows are cswap's last good reading, not a current one.
    pub stale: bool,
    pub windows: Vec<Window>,
}

impl ClaudeAccount {
    pub fn name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.email)
    }

    /// Why the windows are missing or old, in cswap's own terms.
    pub fn note(&self) -> Option<&'static str> {
        match self.status {
            ClaudeAccountStatus::Ok if self.stale => Some("last known"),
            ClaudeAccountStatus::Ok => None,
            ClaudeAccountStatus::TokenExpired => Some("token expired"),
            ClaudeAccountStatus::ApiKey => Some("API key"),
            ClaudeAccountStatus::KeychainUnavailable => Some("keychain unavailable"),
            ClaudeAccountStatus::ReloginRequired => Some("re-login needed"),
            ClaudeAccountStatus::ForeignCredential => Some("foreign credential"),
            ClaudeAccountStatus::NoCredentials => Some("no credentials"),
            ClaudeAccountStatus::Unavailable | ClaudeAccountStatus::Unknown => Some("unavailable"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeAccountStatus {
    Ok,
    TokenExpired,
    ApiKey,
    KeychainUnavailable,
    ReloginRequired,
    ForeignCredential,
    NoCredentials,
    Unavailable,
    #[serde(other)]
    Unknown,
}
