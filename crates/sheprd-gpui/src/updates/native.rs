// ABOUTME: Binds the signed preview's Info.plist policy to Sparkle's safe wrapper.
// ABOUTME: Retains its controller on the AppKit main thread without replacing quit hooks.
use super::{
    Error,
    policy::{self, Bundle},
};
use gpui::{App, Global};
use objc2_foundation::{NSBundle, NSNumber, NSString};
use sparkle_updater::{MainThreadMarker, SparkleUpdater, UpdaterConfig};

struct Updates(SparkleUpdater);
impl Global for Updates {}

#[derive(serde::Deserialize)]
struct Trust {
    feed_url: String,
    public_key: String,
}

fn string(bundle: &NSBundle, key: &str) -> String {
    bundle
        .objectForInfoDictionaryKey(&NSString::from_str(key))
        .and_then(|value| value.downcast::<NSString>().ok())
        .map_or_else(String::new, |value| value.to_string())
}

fn boolean(bundle: &NSBundle, key: &str) -> Option<bool> {
    bundle
        .objectForInfoDictionaryKey(&NSString::from_str(key))
        .and_then(|value| value.downcast::<NSNumber>().ok())
        .map(|value| value.boolValue())
}

pub(super) fn install(cx: &mut App) -> Result<(), Error> {
    if !super::supported_build() {
        return Ok(());
    }
    let Some(mtm) = MainThreadMarker::new() else {
        return Err(Error::Unsupported);
    };
    let trust: Trust =
        serde_json::from_str(include_str!("../../../../scripts/release/sparkle.json"))?;
    let bundle = NSBundle::mainBundle();
    let id = string(&bundle, "CFBundleIdentifier");
    let version = string(&bundle, "CFBundleVersion");
    let path = bundle.bundlePath().to_string();
    let feed = string(&bundle, "SUFeedURL");
    let key = string(&bundle, "SUPublicEDKey");
    let actual = Bundle {
        id: &id,
        version: &version,
        path: &path,
        feed: &feed,
        key: &key,
        signed_feed: boolean(&bundle, "SURequireSignedFeed") == Some(true),
        verify_before_extraction: boolean(&bundle, "SUVerifyUpdateBeforeExtraction") == Some(true),
        automatic_install: boolean(&bundle, "SUAutomaticallyUpdate") != Some(false),
        automatic_checks_configured: bundle
            .objectForInfoDictionaryKey(&NSString::from_str("SUEnableAutomaticChecks"))
            .is_some(),
    };
    if !policy::eligible(
        &actual,
        crate::constants::APP_ID,
        crate::constants::APP_VERSION,
        &trust.feed_url,
        &trust.public_key,
        env!("HERDR_BUILD_WORKTREE"),
    ) {
        return Err(Error::Unsupported);
    }
    // Clear a stored feed override before startup so automatic checks use the signed bundle feed.
    objc2_foundation::NSUserDefaults::standardUserDefaults()
        .removeObjectForKey(&NSString::from_str("SUFeedURL"));
    if let Some(updater) = SparkleUpdater::new(mtm, UpdaterConfig::default())? {
        // AppKit termination invokes GPUI's existing on_quit -> shutdown observers.
        // Do not install a second delegate or call process::exit on relaunch.
        cx.set_global(Updates(updater));
    }
    Ok(())
}

pub(super) fn available(cx: &App) -> Result<(), Error> {
    let updater = cx.try_global::<Updates>().ok_or(Error::Unsupported)?;
    if !updater.0.can_check_for_updates()? {
        return Err(Error::Busy);
    }
    Ok(())
}

pub(super) fn check(cx: &App) -> Result<(), Error> {
    cx.try_global::<Updates>()
        .ok_or(Error::Unsupported)?
        .0
        .check_for_updates()?;
    Ok(())
}
