// ABOUTME: Select the identity a build is compiled with: development, macOS preview or production.
// ABOUTME: No build takes a signed identity unless its build script asked for one.

/// The channel of an ordinary development build.
pub const DEVELOPMENT: &str = "development";
/// The channel of the signed macOS preview route.
pub const MACOS_PREVIEW: &str = "macos-preview";
/// The channel of a signed macOS production release.
pub const PRODUCTION: &str = "production";

/// Preview and production ship as one signed app identity, so the bundle ID the
/// OS sees never decides which channel a copy follows; the channel does.
const SIGNED_APP_ID: &str = include_str!("macos-preview-app-id");

/// What a build was asked to be: the one identity request its build script read.
#[derive(Debug, PartialEq, Eq)]
pub enum Requested<'a> {
    Development,
    MacosPreview(&'a str),
    Production(&'a str),
}

/// What a build is: its channel name, the app ID the OS sees, and its version.
#[derive(Debug, PartialEq, Eq)]
pub struct Identity<'a> {
    pub channel: &'static str,
    pub app_id: &'static str,
    pub version: &'a str,
}

/// The only place that decides a build's channel and identity.
pub fn select<'a>(requested: Requested<'a>, target_os: &str) -> Option<Identity<'a>> {
    let (channel, version) = match requested {
        Requested::Development => (DEVELOPMENT, None),
        Requested::MacosPreview(version) => (MACOS_PREVIEW, Some(version)),
        Requested::Production(version) => (PRODUCTION, Some(version)),
    };
    let Some(version) = version else {
        return Some(Identity {
            channel,
            app_id: app_id(include_str!("development-app-id"))?,
            version: "dev",
        });
    };
    if target_os != "macos" || !signed_version(version) {
        return None;
    }
    Some(Identity {
        channel,
        app_id: app_id(SIGNED_APP_ID)?,
        version,
    })
}

/// A signed release version, `YYYYMMDD.COUNTER`: every part positive, no
/// leading zeros, nothing around it. Previews and production share it so one
/// version orders both channels.
fn signed_version(version: &str) -> bool {
    let Some((date, counter)) = version.split_once('.') else {
        return false;
    };
    let positive_digits = |part: &str| {
        !part.starts_with('0') && !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())
    };
    date.len() == 8 && positive_digits(date) && positive_digits(counter)
}

/// Identity data files allow a conventional final newline, but no empty segments
/// or characters that could become a second build-script directive.
pub fn app_id(value: &str) -> Option<&str> {
    let value = value.trim();
    (value.contains('.')
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        }))
    .then_some(value)
}
