// ABOUTME: Probes real GNOME startup in a private session on a disposable Linux runner.
// ABOUTME: Runs explicitly selected GTK and Spaceport tests on its owned display.
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fs,
    io::{self, Read},
    os::unix::fs::{DirBuilderExt, FileTypeExt},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[path = "../../crates/test-support/linux_shortcuts/receipt.rs"]
mod receipt;

#[cfg(test)]
#[path = "linux_gnome_probe/tests.rs"]
mod tests;

struct Owned(Child);

enum Probe {
    Startup,
    Gtk(PathBuf),
    Spaceport {
        gtk: PathBuf,
        test: PathBuf,
        herdr: PathBuf,
    },
}

impl Owned {
    fn reap(&mut self) -> io::Result<ExitStatus> {
        if self.0.try_wait()?.is_none() {
            self.0.kill()?;
        }
        self.0.wait()
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

fn wait(child: &mut Child, duration: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + duration;
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "owned child exceeded its deadline",
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn command(program: impl AsRef<OsStr>, root: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_RUNTIME_DIR", root.join("runtime"))
        .stdin(Stdio::null())
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", root.join("bus").display()),
        )
        .current_dir(root);
    command
}

fn input_probe(root: &Path, script: &Path, display: &Path) -> io::Result<()> {
    let log = fs::File::create(root.join("input.log"))?;
    let mut driver = Owned(
        command("gjs", root)
            .arg("-m")
            .arg(script)
            .env("WAYLAND_DISPLAY", display)
            .env("GDK_BACKEND", "wayland")
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?,
    );
    let result = wait(&mut driver.0, Duration::from_secs(45));
    let cleanup = driver.reap();
    println!("GNOME_PROBE input_driver_reaped={}", cleanup.is_ok());
    print_log(root, "input.log")?;
    let status = result?;
    cleanup?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "native input prerequisite failed: {status}"
        )));
    }
    Ok(())
}

fn native_test(root: &Path, display: &Path, binary: &Path, herdr: &Path) -> io::Result<()> {
    let log = fs::File::create(root.join("spaceport.log"))?;
    let mut test = Owned(
        command(binary, root)
            .args([
                "--ignored",
                "--exact",
                "linux_shortcuts::native_linux_shortcuts",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("WAYLAND_DISPLAY", display)
            .env("HERDR_TEST_BINARY", herdr)
            .env("HERDR_TEST_TMPDIR", "/tmp")
            .env("GITHUB_ACTIONS", "true")
            .env("RUNNER_ENVIRONMENT", "github-hosted")
            .env("RUNNER_OS", "Linux")
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?,
    );
    let result = wait(&mut test.0, Duration::from_secs(180));
    let cleanup = test.reap();
    println!("GNOME_PROBE native_test_reaped={}", cleanup.is_ok());
    if result.is_err() {
        eprintln!(
            "GNOME_PROBE native test exceeded the outer bound; descendant containment remains the disposable runner's responsibility"
        );
    }
    let mut log = String::new();
    fs::File::open(root.join("spaceport.log"))?
        .take(1_048_577)
        .read_to_string(&mut log)?;
    if log.len() > 1_048_576 {
        return Err(io::Error::other("native test output exceeds 1 MiB"));
    }
    for line in native_receipts(&log) {
        println!(
            "GNOME_RECEIPT {}",
            line.chars().take(600).collect::<String>()
        );
    }
    print_tail("spaceport.log", &log);
    let status = result?;
    cleanup?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "Spaceport native pipeline failed: {status}"
        )));
    }
    require_native_verdict(&log)?;
    Ok(())
}

fn native_receipts(log: &str) -> impl Iterator<Item = &str> {
    let mut seen = HashSet::new();
    log.lines().filter(move |line| {
        ["LINUX_NATIVE ", "LINUX_KEYS ", "LINUX_KEYS_CASE "]
            .iter()
            .any(|prefix| line.starts_with(prefix))
            && seen.insert(*line)
    })
}

fn require_native_verdict(log: &str) -> io::Result<()> {
    if log.lines().any(|line| line == receipt::NATIVE_PASS) {
        Ok(())
    } else {
        Err(io::Error::other(
            "native test ended without its confirmed PASS receipt",
        ))
    }
}

fn session(root: &Path, probe: &Probe) -> io::Result<()> {
    let log = fs::File::create(root.join("gnome.log"))?;
    let mut shell = Owned(
        command("gnome-shell", root)
            .args([
                "--headless",
                "--wayland",
                "--no-x11",
                "--virtual-monitor",
                "1280x800",
                "--sm-disable",
            ])
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?,
    );
    let result = (|| {
        let deadline = Instant::now() + Duration::from_secs(25);
        loop {
            if let Some(status) = shell.0.try_wait()? {
                return Err(io::Error::other(format!(
                    "owned GNOME Shell exited: {status}"
                )));
            }
            let mut display = None;
            for entry in fs::read_dir(root.join("runtime"))? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with("wayland-")
                    && entry.file_type()?.is_socket()
                {
                    display = Some(entry.path());
                }
            }
            let output = fs::File::create(root.join("bus-owner.log"))?;
            let mut query = Owned(
                command("gdbus", root)
                    .args([
                        "call",
                        "--session",
                        "--dest",
                        "org.freedesktop.DBus",
                        "--object-path",
                        "/org/freedesktop/DBus",
                        "--method",
                        "org.freedesktop.DBus.GetNameOwner",
                        "org.gnome.Shell",
                    ])
                    .stdout(Stdio::from(output.try_clone()?))
                    .stderr(Stdio::from(output))
                    .spawn()?,
            );
            let owner = wait(&mut query.0, Duration::from_secs(2))?;
            if let Some(display) = display
                && owner.success()
                && shell.0.try_wait()?.is_none()
            {
                println!(
                    "GNOME_PROBE startup=true private_wayland=true private_bus_owner=true spaceport_started=false keyboard_input=false"
                );
                return match probe {
                    Probe::Startup => Ok(()),
                    Probe::Gtk(script) => input_probe(root, script, &display),
                    Probe::Spaceport { gtk, test, herdr } => {
                        input_probe(root, gtk, &display)?;
                        native_test(root, &display, test, herdr)
                    }
                };
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "GNOME startup did not publish its display and bus owner",
                ));
            }
            thread::sleep(Duration::from_millis(100));
        }
    })();
    let cleanup = shell.reap();
    println!("GNOME_PROBE shell_reaped={}", cleanup.is_ok());
    if let Err(error) = &cleanup {
        eprintln!("GNOME_PROBE shell cleanup failed: {error}");
    }
    print_log(root, "gnome.log")?;
    result.and(cleanup.map(|_| ()))
}

fn print_log(root: &Path, name: &str) -> io::Result<()> {
    let mut log = String::new();
    fs::File::open(root.join(name))?
        .take(32768)
        .read_to_string(&mut log)?;
    print_tail(name, &log);
    Ok(())
}

fn print_tail(name: &str, log: &str) {
    println!("GNOME_LOG {name} bounded prefix follows");
    for line in log
        .lines()
        .rev()
        .take(30)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        println!("GNOME_LOG {}", line.chars().take(600).collect::<String>());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (key, expected) in [
        ("GITHUB_ACTIONS", "true"),
        ("RUNNER_ENVIRONMENT", "github-hosted"),
        ("RUNNER_OS", "Linux"),
    ] {
        if env::var(key).as_deref() != Ok(expected) {
            return Err(
                io::Error::other("requires a disposable GitHub-hosted Linux runner").into(),
            );
        }
    }
    let args = env::args_os()
        .skip(1)
        .map(fs::canonicalize)
        .collect::<io::Result<Vec<_>>>()?;
    let probe = match args.as_slice() {
        [] if cfg!(target_os = "linux") => Probe::Startup,
        [gtk] if cfg!(target_os = "linux") => Probe::Gtk(gtk.clone()),
        [gtk, test, herdr] if cfg!(target_os = "linux") => Probe::Spaceport {
            gtk: gtk.clone(),
            test: test.clone(),
            herdr: herdr.clone(),
        },
        _ => {
            return Err(io::Error::other(
                "usage: spaceport-gnome-probe [GTK_SCRIPT [LIVE_GUI_TEST HERDR_BINARY]]",
            )
            .into());
        }
    };
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("spaceport-gnome-{}-{nonce}", std::process::id()));
    fs::DirBuilder::new().mode(0o700).create(&root)?;
    for name in ["home", "config", "data", "cache", "state", "runtime"] {
        fs::DirBuilder::new().mode(0o700).create(root.join(name))?;
    }
    println!("GNOME_PROBE private_root={}", root.display());
    let log = fs::File::create(root.join("bus.log"))?;
    let mut bus = Owned(
        command("dbus-daemon", &root)
            .args(["--session", "--nofork", "--nosyslog"])
            .arg(format!(
                "--address=unix:path={}",
                root.join("bus").display()
            ))
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .spawn()?,
    );
    let result = (|| {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(status) = bus.0.try_wait()? {
                return Err(io::Error::other(format!(
                    "owned session bus exited: {status}"
                )));
            }
            match fs::symlink_metadata(root.join("bus")) {
                Ok(metadata) if metadata.file_type().is_socket() => break,
                Ok(_) => return Err(io::Error::other("private bus path is not a socket")),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "private bus startup expired",
                ));
            }
            thread::sleep(Duration::from_millis(20));
        }
        session(&root, &probe)
    })();
    let cleanup = bus.reap();
    println!(
        "GNOME_PROBE bus_reaped={} root_preserved_for_runner_cleanup={}",
        cleanup.is_ok(),
        root.display()
    );
    if let Err(error) = &cleanup {
        eprintln!("GNOME_PROBE bus cleanup failed: {error}");
    }
    print_log(&root, "bus.log")?;
    result.and(cleanup.map(|_| ()))?;
    Ok(())
}
