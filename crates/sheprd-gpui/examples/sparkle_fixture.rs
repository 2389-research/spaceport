// ABOUTME: Run the real Sparkle native UI in a signed, isolated GPUI fixture.
// ABOUTME: Check signed sandbox metadata before startup and retain main-thread ownership.
#[cfg(target_os = "macos")]
mod native {
    use anyhow::{Context as _, Result, bail};
    use gpui::{App, Context, Global, IntoElement, Render, Window, WindowOptions, div, prelude::*};
    use objc2_foundation::{
        NSBundle, NSDate, NSHomeDirectory, NSSearchPathDirectory, NSSearchPathDomainMask,
        NSSearchPathForDirectoriesInDomains, NSString, NSTemporaryDirectory, NSUserDefaults,
    };
    use serde_json::json;
    use sparkle_updater::{MainThreadMarker, SparkleUpdater, UpdaterConfig};
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        os::unix::fs::MetadataExt as _,
        path::{Path, PathBuf},
        rc::Rc,
        sync::mpsc::{SyncSender, sync_channel},
        time::Duration,
    };

    enum CheckMode {
        Information,
        Manual,
    }

    struct Fixture(SparkleUpdater);
    impl Global for Fixture {}

    struct FixtureWindow;
    impl Render for FixtureWindow {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .p_8()
                .child("Isolated Sparkle fixture — use the native update dialog.")
        }
    }

    fn string(bundle: &NSBundle, name: &str) -> String {
        bundle
            .objectForInfoDictionaryKey(&NSString::from_str(name))
            .and_then(|value| value.downcast::<NSString>().ok())
            .map_or_else(String::new, |value| value.to_string())
    }

    fn within(root: &Path, candidate: impl AsRef<Path>) -> Result<PathBuf> {
        let candidate = candidate.as_ref().canonicalize()?;
        if candidate == root || !candidate.starts_with(root) {
            bail!("fixture path escapes signed root: {}", candidate.display());
        }
        Ok(candidate)
    }

    fn isolation(bundle: &NSBundle) -> Result<PathBuf> {
        let id = string(bundle, "CFBundleIdentifier");
        if !id.starts_with("ai.2389.spaceport.fixture.")
            || id != string(bundle, "SpaceportFixtureID")
        {
            bail!("only explicit fixture bundles may run this example");
        }
        let root = PathBuf::from(string(bundle, "SpaceportFixtureRoot")).canonicalize()?;
        let home = within(&root, root.join("home"))?;
        if PathBuf::from(NSHomeDirectory().to_string()).canonicalize()? != home {
            bail!("Foundation HOME does not match signed fixture HOME");
        }
        for key in ["HOME", "CFFIXED_USER_HOME"] {
            if PathBuf::from(std::env::var(key)?).canonicalize()? != home {
                bail!("{key} does not match signed fixture HOME");
            }
        }
        let native_temp = PathBuf::from(NSTemporaryDirectory().to_string()).canonicalize()?;
        let pinned_temp =
            PathBuf::from(string(bundle, "SpaceportFixtureNativeTemp")).canonicalize()?;
        let metadata = fs::metadata(&native_temp)?;
        let owner = fs::metadata(&root)?.uid();
        // Foundation uses Darwin's protected per-user scratch even with a private TMPDIR.
        // Pin that exact native root; never scan or clean its other contents.
        if (native_temp != pinned_temp && native_temp != root.join("tmp").canonicalize()?)
            || metadata.uid() != owner
            || metadata.mode() & 0o777 != 0o700
            || string(bundle, "SpaceportFixtureOwner") != owner.to_string()
        {
            bail!("Foundation temp does not match the pinned private owner/path");
        }
        within(&root, std::env::var("TMPDIR")?)?;
        for directory in [
            NSSearchPathDirectory::CachesDirectory,
            NSSearchPathDirectory::LibraryDirectory,
        ] {
            let paths = NSSearchPathForDirectoriesInDomains(
                directory,
                NSSearchPathDomainMask::UserDomainMask,
                true,
            );
            if paths.is_empty() {
                bail!("Foundation returned no fixture directory");
            }
            for path in paths.iter() {
                within(&root, path.to_string())?;
            }
        }
        for key in [
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "XDG_CACHE_HOME",
            "XDG_RUNTIME_DIR",
        ] {
            within(&root, std::env::var(key)?)?;
        }
        within(&root, bundle.bundlePath().to_string())?;
        within(&root, std::env::current_exe()?)?;
        Ok(root)
    }

    fn seed_feed_failure(bundle: &NSBundle, first_launch: bool) -> Result<bool> {
        if !first_launch {
            return Ok(false);
        }
        match string(bundle, "SpaceportFixtureFailedFeedAgeDays").as_str() {
            "0" => return Ok(false),
            "21" => {}
            _ => bail!("signed fixture feed failure age is invalid"),
        }
        // SUAppcastDriver.m uses this NSDate in the host's exact preferences domain.
        // isolation() already proved the signed unique fixture ID and private paths.
        let defaults = NSUserDefaults::standardUserDefaults();
        let key = NSString::from_str("SUInitialFailedFeedSigningValidationDate");
        if defaults.objectForKey(&key).is_some() {
            bail!("fixture feed failure date already exists");
        }
        let date = NSDate::dateWithTimeIntervalSinceNow(-21.0 * 86_400.0);
        // SAFETY: Retained NSDate/NSString are valid property-list objects; this
        // main-thread call uses only the already-validated unique fixture domain.
        // Primitive setters cannot store NSDate; domain-replacement APIs are also unsafe.
        #[allow(
            unsafe_code,
            reason = "Foundation requires a type proof to store NSDate"
        )]
        unsafe {
            defaults.setObject_forKey(Some(&date), &key)
        };
        let recorded = defaults
            .objectForKey(&key)
            .and_then(|value| value.downcast::<NSDate>().ok())
            .context("fixture failed to store an NSDate")?;
        if recorded.timeIntervalSinceNow() > -20.0 * 86_400.0 {
            bail!("fixture failed-signature date is not older than 20 days");
        }
        Ok(true)
    }

    fn writer(root: &Path) -> Result<SyncSender<serde_json::Value>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join("events.jsonl"))?;
        let (sender, receiver) = sync_channel::<serde_json::Value>(256);
        std::thread::Builder::new()
            .name("fixture-evidence".into())
            .spawn(move || {
                let mut written = 0;
                for record in receiver {
                    let line = record.to_string();
                    written += line.len() + 1;
                    if written > 1024 * 1024 {
                        eprintln!("fixture evidence exceeded 1 MiB");
                        break;
                    }
                    if writeln!(file, "{line}").is_err() {
                        eprintln!("fixture evidence write failed");
                        break;
                    }
                }
            })?;
        Ok(sender)
    }

    fn watch_quit(cx: &App, root: PathBuf, sender: SyncSender<serde_json::Value>) {
        let executor = cx.background_executor().clone();
        cx.spawn(async move |cx| {
            loop {
                executor.timer(Duration::from_millis(100)).await;
                let path = root.join("quit");
                let requested = executor
                    .spawn(async move {
                        match fs::symlink_metadata(&path) {
                            Ok(metadata)
                                if metadata.is_file() && metadata.mode() & 0o777 == 0o600 =>
                            {
                                fs::read_to_string(path).map(|contents| contents == "quit\n")
                            }
                            Ok(_) => Ok(false),
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                            Err(error) => Err(error),
                        }
                    })
                    .await;
                match requested {
                    Ok(true) => {
                        let _ = sender.try_send(
                            json!({"event": "fixture://quit-requested", "pid": std::process::id()}),
                        );
                        cx.update(|cx| cx.quit());
                        break;
                    }
                    Ok(false) => {}
                    Err(error) => {
                        eprintln!("fixture quit sentinel check failed: {error}");
                        break;
                    }
                }
            }
        })
        .detach();
    }

    fn install(
        cx: &mut App,
        sender: SyncSender<serde_json::Value>,
        first_launch: bool,
        mode: CheckMode,
    ) -> Result<()> {
        let mtm = MainThreadMarker::new().context("fixture requires the main thread")?;
        let ready = sender.clone();
        let updater = SparkleUpdater::new(mtm, UpdaterConfig {
        event_callback: Some(Rc::new(move |event| {
            if sender.try_send(json!({"event": event.name(), "payload": event.payload(), "pid": std::process::id()})).is_err() {
                eprintln!("fixture evidence channel full or closed");
            }
        })),
        ..Default::default()
    })?.context("fixture must run inside its signed bundle")?;
        cx.set_global(Fixture(updater));
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| FixtureWindow))?;
        cx.activate(true);
        ready.try_send(json!({"event": "fixture://ready", "pid": std::process::id()}))?;
        if first_launch {
            match mode {
                CheckMode::Information => {
                    cx.global::<Fixture>().0.check_for_update_information()?
                }
                CheckMode::Manual => cx.global::<Fixture>().0.check_for_updates()?,
            }
        }
        Ok(())
    }

    pub(super) fn run() -> Result<()> {
        let bundle = NSBundle::mainBundle();
        // Before the UI loop, synchronous filesystem checks cannot stall UI callbacks.
        let root = isolation(&bundle)?;
        let first_launch = fs::read_dir(root.join("launches"))?.next().is_none();
        let aged_feed_failure_seeded = seed_feed_failure(&bundle, first_launch)?;
        let marker = json!({
            "aged_feed_failure_seeded": aged_feed_failure_seeded,
            "pid": std::process::id(),
            "executable": std::env::current_exe()?.canonicalize()?,
            "bundle_id": string(&bundle, "CFBundleIdentifier"),
            "version": string(&bundle, "CFBundleVersion"),
            "home": NSHomeDirectory().to_string(),
            "tmp": NSTemporaryDirectory().to_string(),
        });
        let mut launch = OpenOptions::new().write(true).create_new(true).open(
            root.join("launches")
                .join(format!("{}.json", std::process::id())),
        )?;
        writeln!(launch, "{marker}")?;
        let mode = match string(&bundle, "SpaceportFixtureCheckMode").as_str() {
            "information" => CheckMode::Information,
            "manual" => CheckMode::Manual,
            _ => bail!("signed fixture check mode is invalid"),
        };
        let sender = writer(&root)?;
        gpui_platform::application().run(move |cx| {
            watch_quit(cx, root, sender.clone());
            if let Err(error) = install(cx, sender, first_launch, mode) {
                eprintln!("fixture startup failed: {error:#}");
                cx.quit();
            }
        });
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn main() -> anyhow::Result<()> {
    native::run()
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("The signed Sparkle native fixture requires macOS.");
    std::process::exit(1);
}
