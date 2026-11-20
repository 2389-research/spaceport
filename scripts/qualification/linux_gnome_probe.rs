// ABOUTME: Probes real GNOME startup in a private session on a disposable Linux runner.
// ABOUTME: This throwaway prerequisite probe starts no Spaceport process and sends no keys.
use std::{
    env, fs,
    io::{self, Read},
    os::unix::fs::{DirBuilderExt, FileTypeExt},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Owned(Child);

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

fn command(program: &str, root: &Path) -> Command {
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
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}", root.join("bus").display()),
        )
        .current_dir(root);
    command
}

fn session(root: &Path) -> io::Result<()> {
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
            let mut socket = false;
            for entry in fs::read_dir(root.join("runtime"))? {
                let entry = entry?;
                socket |= entry.file_name().to_string_lossy().starts_with("wayland-")
                    && entry.file_type()?.is_socket();
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
            if socket && owner.success() && shell.0.try_wait()?.is_none() {
                println!(
                    "GNOME_PROBE startup=true private_wayland=true private_bus_owner=true spaceport_started=false keyboard_input=false"
                );
                return Ok(());
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
    Ok(())
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
    if !cfg!(target_os = "linux") || env::args_os().len() != 1 {
        return Err(io::Error::other("usage: spaceport-gnome-probe").into());
    }
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
        session(&root)
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
