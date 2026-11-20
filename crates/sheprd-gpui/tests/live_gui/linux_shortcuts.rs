// ABOUTME: Qualifies fixed shortcuts through an isolated GNOME Wayland desktop.
// ABOUTME: Owns native input, private layout changes, and exact fixture children.

use super::{Isolated, Sandbox, daemon_binary, fixture_log_tail, gui_command};
use anyhow::{Context as _, Result, anyhow, bail, ensure};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::Read as _,
    os::unix::fs::{FileTypeExt as _, MetadataExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use zbus::zvariant::{DynamicDeserialize, DynamicType, OwnedObjectPath, OwnedValue, Value};

#[path = "../../../test-support/linux_shortcuts.rs"]
mod cases;
use cases::{CASES, Notice};

#[path = "../../../test-support/linux_shortcuts/receipt.rs"]
mod receipt;

const CALL_TIMEOUT: Duration = Duration::from_secs(2);
const PHASE_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_LOG: u64 = 256 * 1024;
const MANAGER: &str = "org.gnome.Mutter.RemoteDesktop";
const MANAGER_PATH: &str = "/org/gnome/Mutter/RemoteDesktop";
const SESSION: &str = "org.gnome.Mutter.RemoteDesktop.Session";
const SOURCE_SCHEMA: &str = "org.gnome.desktop.input-sources";

#[test]
#[ignore = "requires disposable GNOME Wayland desktop and explicit HERDR_TEST_BINARY"]
fn native_linux_shortcuts() {
    run().expect("native Linux shortcut qualification failed");
}

fn run() -> Result<()> {
    let desktop = PrivateDesktop::read()?;
    let binary = daemon_binary();
    let mut isolated = Isolated {
        sandbox: Sandbox::new(),
        daemon: None,
        gui: None,
    };
    let logs = isolated.sandbox.dir.clone();
    fs::write(
        isolated.sandbox.dir.join(cases::CAPTURE_FILE),
        include_str!("../../../test-support/linux_shortcuts/capture.sh"),
    )
    .context("writing owned terminal byte capture")?;
    let mut layout = Layout::capture(&desktop, &logs)?;
    let mut native = None;
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
        layout.polish()?;
        isolated.daemon = Some(
            isolated
                .sandbox
                .command(binary, "daemon.log")
                .arg("server")
                .spawn()
                .context("starting private Herdr")?,
        );
        isolated.sandbox.wait_for_daemon(
            isolated.daemon.as_mut().context("missing owned daemon")?,
            Duration::from_secs(20),
        );
        isolated.gui = Some(
            gui_command(&isolated.sandbox, |name| std::env::var_os(name))
                .env("DBUS_SESSION_BUS_ADDRESS", &desktop.bus)
                .env("HERDR_TEST_LINUX_SHORTCUTS", &nonce)
                .args([
                    OsStr::new("--socket"),
                    isolated.sandbox.socket().as_os_str(),
                ])
                .arg("--integration-test")
                .spawn()
                .context("starting private Spaceport")?,
        );
        let deadline = Instant::now() + Duration::from_secs(90);
        let mut observer = Observer {
            path: isolated.sandbox.dir.join("gui.log"),
            nonce: &nonce,
            cursor: 0,
            deadline,
        };
        observer.wait(Notice::Mapped, &mut isolated)?;
        native = Some(NativeMutter::connect(&desktop.bus)?);
        let input = native.as_mut().context("missing native input owner")?;
        input.prepare_shell(deadline)?;
        input.start()?;
        input.chord(&[56, 15])?;
        for case in CASES {
            if case.outcome == cases::Outcome::CycleWindows {
                observer.wait(Notice::FocusOriginal(case.id), &mut isolated)?;
                // This prepares focus through GNOME; READY still requires the exact original window.
                input.chord(&[56, 1])?;
                eprintln!("LINUX_NATIVE preparation=original-window native_alt_escape=true");
            }
            observer.wait(Notice::Ready(case.id), &mut isolated)?;
            for chord in case.chords {
                input.chord(chord)?;
            }
            observer.wait(Notice::Passed(case.id), &mut isolated)?;
            eprintln!(
                "LINUX_NATIVE case={} outcome={:?} native_evdev=true",
                case.id, case.outcome
            );
        }
        observer.wait(Notice::Complete, &mut isolated)?;
        let gui = isolated.gui.as_mut().context("missing owned GUI")?;
        let status = wait_child(gui, deadline.min(Instant::now() + PHASE_TIMEOUT))?;
        ensure!(status.success(), "GUI failed: {status}");
        ensure!(
            isolated
                .daemon
                .as_mut()
                .context("missing owned daemon")?
                .try_wait()?
                .is_none(),
            "GUI exit stopped the private daemon"
        );
        Ok(())
    }))
    .unwrap_or_else(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("unknown panic");
        Err(anyhow!("native fixture panicked: {message}"))
    });
    // Keep cleanup independent: one failure must not suppress another owner's cleanup.
    let input_cleanup = native.as_mut().map_or(Ok(()), NativeMutter::cleanup);
    drop(native.take());
    let layout_cleanup = layout.restore();
    super::stop_children([&mut isolated.gui, &mut isolated.daemon]);
    let children_cleanup = (|| -> Result<()> {
        for child in [&mut isolated.gui, &mut isolated.daemon]
            .into_iter()
            .flatten()
        {
            ensure!(child.try_wait()?.is_some(), "owned child survived cleanup");
        }
        Ok(())
    })();
    let log = read_bounded(&isolated.sandbox.dir.join("gui.log"), MAX_LOG);
    if let Ok(log) = &log {
        eprintln!("{}", fixture_log_tail(log));
    }
    for (label, outcome) in [
        ("qualification", &result),
        ("native input cleanup", &input_cleanup),
        ("layout cleanup", &layout_cleanup),
        ("owned child cleanup", &children_cleanup),
    ] {
        if let Err(error) = outcome {
            eprintln!("LINUX_NATIVE {label} FAIL: {error:#}");
        }
    }
    result?;
    input_cleanup?;
    layout_cleanup?;
    children_cleanup?;
    log?;
    eprintln!("{}", receipt::NATIVE_PASS);
    Ok(())
}

struct PrivateDesktop {
    root: PathBuf,
    bus: String,
}

impl PrivateDesktop {
    fn read() -> Result<Self> {
        for (name, expected) in [
            ("GITHUB_ACTIONS", "true"),
            ("RUNNER_ENVIRONMENT", "github-hosted"),
            ("RUNNER_OS", "Linux"),
        ] {
            ensure!(
                std::env::var(name).as_deref() == Ok(expected),
                "requires disposable GitHub-hosted Linux runner"
            );
        }
        let home = PathBuf::from(std::env::var_os("HOME").context("missing private HOME")?);
        ensure!(
            home.is_absolute() && home.file_name() == Some(OsStr::new("home")),
            "unexpected private HOME"
        );
        let root = home
            .parent()
            .context("private HOME has no parent")?
            .to_owned();
        ensure!(
            root.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("spaceport-gnome-")),
            "unexpected GNOME private root"
        );
        let metadata = fs::symlink_metadata(&root)?;
        ensure!(
            metadata.is_dir() && metadata.permissions().mode() & 0o777 == 0o700,
            "GNOME root must be a private directory"
        );
        ensure!(
            fs::canonicalize(&root)? == root,
            "private GNOME root contains a path alias"
        );
        for (name, directory) in [
            ("HOME", "home"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_DATA_HOME", "data"),
            ("XDG_CACHE_HOME", "cache"),
            ("XDG_STATE_HOME", "state"),
            ("XDG_RUNTIME_DIR", "runtime"),
        ] {
            ensure!(
                std::env::var_os(name).as_deref() == Some(root.join(directory).as_os_str()),
                "{name} is outside the private GNOME root"
            );
            let directory = fs::symlink_metadata(root.join(directory))?;
            ensure!(
                directory.is_dir()
                    && directory.permissions().mode() & 0o777 == 0o700
                    && directory.uid() == metadata.uid(),
                "{name} is not an owned private directory"
            );
        }
        let bus = std::env::var("DBUS_SESSION_BUS_ADDRESS").context("missing private bus")?;
        ensure!(
            bus == format!("unix:path={}", root.join("bus").display()),
            "private bus must have one explicit filesystem address"
        );
        ensure!(
            fs::symlink_metadata(root.join("bus"))?
                .file_type()
                .is_socket(),
            "private bus path is not a socket"
        );
        let display =
            PathBuf::from(std::env::var_os("WAYLAND_DISPLAY").context("missing Wayland display")?);
        ensure!(
            display.is_absolute() && display.parent() == Some(root.join("runtime").as_path()),
            "Wayland display is outside the private runtime"
        );
        ensure!(
            fs::symlink_metadata(display)?.file_type().is_socket(),
            "Wayland display is not a socket"
        );
        Ok(Self { root, bus })
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LANG", "C.UTF-8")
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_DATA_HOME", self.root.join("data"))
            .env("XDG_CACHE_HOME", self.root.join("cache"))
            .env("XDG_STATE_HOME", self.root.join("state"))
            .env("XDG_RUNTIME_DIR", self.root.join("runtime"))
            .env("DBUS_SESSION_BUS_ADDRESS", &self.bus)
            .stdin(Stdio::null())
            .current_dir(&self.root);
        command
    }
}

struct Layout<'a> {
    desktop: &'a PrivateDesktop,
    logs: &'a Path,
    previous: String,
    attempted: bool,
    operation: usize,
}

impl<'a> Layout<'a> {
    fn capture(desktop: &'a PrivateDesktop, logs: &'a Path) -> Result<Self> {
        let mut layout = Self {
            desktop,
            logs,
            previous: String::new(),
            attempted: false,
            operation: 0,
        };
        layout.previous = layout.settings(&["get", SOURCE_SCHEMA, "sources"])?;
        ensure!(
            !layout.previous.is_empty(),
            "private input sources are empty"
        );
        Ok(layout)
    }

    fn settings(&mut self, arguments: &[&str]) -> Result<String> {
        self.operation += 1;
        let output = self.logs.join(format!("layout-{}.out", self.operation));
        let errors = self.logs.join(format!("layout-{}.err", self.operation));
        let mut child = OwnedChild(Some(
            self.desktop
                .command("/usr/bin/gsettings")
                .args(arguments)
                .stdout(File::create(&output)?)
                .stderr(File::create(&errors)?)
                .spawn()
                .context("starting private gsettings")?,
        ));
        let status = wait_child(
            child.0.as_mut().context("missing gsettings child")?,
            Instant::now() + CALL_TIMEOUT,
        )?;
        child.0.take();
        ensure!(
            status.success(),
            "private gsettings failed: {status}: {}",
            read_bounded(&errors, 4096)?
        );
        Ok(read_bounded(&output, 4096)?.trim().to_owned())
    }

    fn polish(&mut self) -> Result<()> {
        self.attempted = true;
        self.settings(&["set", SOURCE_SCHEMA, "sources", "[('xkb', 'pl')]"])?;
        ensure!(
            self.settings(&["get", SOURCE_SCHEMA, "sources"])? == "[('xkb', 'pl')]",
            "private Polish source was not observed"
        );
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        if self.attempted {
            let previous = self.previous.clone();
            self.settings(&["set", SOURCE_SCHEMA, "sources", &previous])?;
            ensure!(
                self.settings(&["get", SOURCE_SCHEMA, "sources"])? == previous,
                "private input sources did not restore"
            );
            self.attempted = false;
        }
        Ok(())
    }
}

struct OwnedChild(Option<Child>);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn wait_child(child: &mut Child, deadline: Instant) -> Result<ExitStatus> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        ensure!(Instant::now() < deadline, "owned child exceeded deadline");
        thread::sleep(Duration::from_millis(20));
    }
}

fn read_bounded(path: &Path, limit: u64) -> Result<String> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "fixture output exceeds {limit} bytes"
    );
    // A concurrent writer may have written only the first bytes of a character.
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

struct Observer<'a> {
    path: PathBuf,
    nonce: &'a str,
    cursor: usize,
    deadline: Instant,
}

impl Observer<'_> {
    fn wait(&mut self, expected: Notice<'_>, isolated: &mut Isolated) -> Result<()> {
        let deadline = self.deadline.min(Instant::now() + PHASE_TIMEOUT);
        loop {
            let log = read_bounded(&self.path, MAX_LOG)?;
            ensure!(self.cursor <= log.len(), "fixture log was truncated");
            for line in log[self.cursor..].split_inclusive('\n') {
                if !line.ends_with('\n') {
                    break;
                }
                self.cursor += line.len();
                if let Some(actual) =
                    cases::parse_notice(line.trim_end_matches(&['\r', '\n'][..]), self.nonce)
                {
                    ensure!(
                        actual == expected,
                        "out-of-order fixture notice: expected {expected:?}, got {actual:?}"
                    );
                    return Ok(());
                }
            }
            ensure!(
                isolated
                    .gui
                    .as_mut()
                    .context("missing GUI")?
                    .try_wait()?
                    .is_none(),
                "GUI exited before {expected:?}: {}",
                fixture_log_tail(&log)
            );
            ensure!(
                isolated
                    .daemon
                    .as_mut()
                    .context("missing daemon")?
                    .try_wait()?
                    .is_none(),
                "daemon exited before {expected:?}"
            );
            ensure!(
                Instant::now() < deadline,
                "timed out waiting for {expected:?}: {}",
                fixture_log_tail(&log)
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
}

struct NativeMutter {
    connection: zbus::Connection,
    session: SessionState,
    held: Vec<u32>,
}

#[derive(Debug)]
enum SessionState {
    NotRequested,
    CreationUnknown,
    Open(OwnedObjectPath),
    CloseUnknown,
    Closed,
}

impl NativeMutter {
    fn connect(address: &str) -> Result<Self> {
        let builder = zbus::connection::Builder::address(address)?.method_timeout(CALL_TIMEOUT);
        // Authentication has no method-call timeout. A cancellable timer bounds it too.
        let (expired, elapsed) = futures_channel::oneshot::channel();
        let (cancel, canceled) = mpsc::channel();
        let timer = thread::spawn(move || {
            if matches!(
                canceled.recv_timeout(CALL_TIMEOUT),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                let _ = expired.send(());
            }
        });
        let connection = futures_lite::future::block_on(futures_lite::future::race(
            async {
                builder
                    .build()
                    .await
                    .context("connecting explicit private bus")
            },
            async {
                elapsed.await.context("private bus timer stopped")?;
                bail!("private bus authentication exceeded deadline")
            },
        ));
        let _ = cancel.send(());
        timer
            .join()
            .map_err(|_| anyhow!("private bus timer panicked"))?;
        let connection = connection?;
        ensure!(
            connection.is_bus() && connection.unique_name().is_some(),
            "private bus authentication did not complete"
        );
        Ok(Self {
            connection,
            session: SessionState::NotRequested,
            held: Vec::new(),
        })
    }

    fn call<B, R>(
        &self,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        body: &B,
    ) -> Result<R>
    where
        B: serde::Serialize + DynamicType,
        R: for<'d> DynamicDeserialize<'d>,
    {
        futures_lite::future::block_on(async {
            let proxy = zbus::Proxy::new(&self.connection, destination, path, interface).await?;
            proxy.call(method, body).await
        })
        .with_context(|| format!("private desktop {interface}.{method}"))
    }

    fn shell_state(&self) -> Result<(String, bool)> {
        let mut properties: HashMap<String, OwnedValue> = self.call(
            "org.gnome.Shell",
            "/org/gnome/Shell",
            "org.freedesktop.DBus.Properties",
            "GetAll",
            &("org.gnome.Shell",),
        )?;
        let mode = String::try_from(properties.remove("Mode").context("Shell Mode absent")?)?;
        let overview = bool::try_from(
            properties
                .remove("OverviewActive")
                .context("Shell OverviewActive absent")?,
        )?;
        Ok((mode, overview))
    }

    fn prepare_shell(&self, overall: Instant) -> Result<()> {
        let (mode, overview) = self.shell_state()?;
        ensure!(mode == "user", "private GNOME Shell is outside user mode");
        if overview {
            self.call::<_, ()>(
                "org.gnome.Shell",
                "/org/gnome/Shell",
                "org.freedesktop.DBus.Properties",
                "Set",
                &("org.gnome.Shell", "OverviewActive", Value::from(false)),
            )?;
        }
        let deadline = overall.min(Instant::now() + PHASE_TIMEOUT);
        loop {
            let (mode, overview) = self.shell_state()?;
            ensure!(mode == "user", "private Shell left user mode");
            if !overview {
                break;
            }
            ensure!(
                Instant::now() < deadline,
                "private overview remained active"
            );
            thread::sleep(Duration::from_millis(20));
        }
        eprintln!("LINUX_NATIVE shell_user=true overview_active=false private_bus=true");
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        ensure!(
            matches!(self.session, SessionState::NotRequested),
            "native session creation was already attempted"
        );
        // A timed-out reply does not establish whether Mutter created a session.
        self.session = SessionState::CreationUnknown;
        let path = self.call(MANAGER, MANAGER_PATH, MANAGER, "CreateSession", &())?;
        self.session = SessionState::Open(path);
        self.call::<_, ()>(MANAGER, self.path()?, SESSION, "Start", &())
    }

    fn path(&self) -> Result<&str> {
        match &self.session {
            SessionState::Open(path) => Ok(path.as_str()),
            state => bail!("native session has no confirmed open path: {state:?}"),
        }
    }

    fn key(&mut self, code: u32, pressed: bool) -> Result<()> {
        if pressed {
            ensure!(
                !self.held.contains(&code),
                "native key already held: {code}"
            );
            self.held.push(code);
        }
        self.call::<_, ()>(
            MANAGER,
            self.path()?,
            SESSION,
            "NotifyKeyboardKeycode",
            &(code, pressed),
        )?;
        if !pressed {
            self.held.retain(|held| *held != code);
        }
        Ok(())
    }

    fn chord(&mut self, chord: &[u32]) -> Result<()> {
        for &code in chord {
            self.key(code, true)?;
        }
        for &code in chord.iter().rev() {
            self.key(code, false)?;
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        let mut failures = Vec::new();
        for code in self.held.clone().into_iter().rev() {
            if let Err(error) = self.key(code, false) {
                failures.push(error);
            }
        }
        if let SessionState::Open(path) = &self.session {
            let path = path.clone();
            self.session = SessionState::CloseUnknown;
            match self.call::<_, ()>(MANAGER, path.as_str(), SESSION, "Stop", &()) {
                Ok(()) => self.session = SessionState::Closed,
                Err(error) => failures.push(error),
            }
        }
        if matches!(
            self.session,
            SessionState::CreationUnknown | SessionState::CloseUnknown
        ) {
            failures.push(anyhow!(
                "native session cleanup is unconfirmed: {:?}; dropping the owned peer",
                self.session
            ));
        }
        ensure!(
            failures.is_empty() && self.held.is_empty(),
            "native cleanup failed: {failures:#?}; held={:?}; session={:?}",
            self.held,
            self.session
        );
        match self.session {
            SessionState::Closed => {
                eprintln!("LINUX_NATIVE keys_released=true session_closed=true")
            }
            SessionState::NotRequested => eprintln!(
                "LINUX_NATIVE keys_released=true session_created=false close_required=false"
            ),
            _ => bail!(
                "native session cleanup state is invalid: {:?}",
                self.session
            ),
        }
        Ok(())
    }
}
