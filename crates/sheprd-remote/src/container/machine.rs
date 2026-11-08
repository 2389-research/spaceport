//! Connection commands for a saved container on the local engine.
//! A container is named by its device ID and keeps its home in a volume.
//!
//! Agents run as an unprivileged `herdr` user.

use super::{Engine, Error, Result, engine::detail};
use std::{process::Command, time::Duration};

/// The user agents run as, and its home, where the volume is mounted.
pub const USER: &str = "herdr";
pub const HOME: &str = "/home/herdr";

const QUICK: Duration = Duration::from_secs(30);

/// Make the `herdr` user own its home, whatever the image ships: Debian and
/// Ubuntu have `useradd`, Alpine and BusyBox images `adduser`. Run again on
/// every start, so it must stay idempotent.
const PREPARE: &str = r#"set -e
if ! id herdr >/dev/null 2>&1; then
  if command -v useradd >/dev/null 2>&1; then
    useradd --home-dir /home/herdr --no-create-home --shell /bin/sh --user-group herdr
  else
    adduser -D -H -h /home/herdr -s /bin/sh herdr
  fi
fi
chown herdr /home/herdr"#;

/// What the engine says about a container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    /// Created, exited, or paused: starting it brings it back with its files.
    Stopped,
    Missing,
}

fn prepare(engine: Engine, id: &str, cancelled: &dyn Fn() -> bool) -> Result<()> {
    engine
        .run(
            "exec",
            ["exec", "--user", "root", id, "/bin/sh", "-c", PREPARE],
            QUICK,
            cancelled,
        )
        .map(drop)
}

pub fn state(engine: Engine, id: &str, cancelled: &dyn Fn() -> bool) -> Result<State> {
    let (code, stdout, stderr) = engine.output(
        "inspect",
        ["container", "inspect", "--format", "{{.State.Status}}", id],
        QUICK,
        cancelled,
    )?;
    if code != Some(0) {
        let detail = detail(&stderr);
        // Docker says "No such container", Podman "no such container".
        if detail.to_ascii_lowercase().contains("no such") {
            return Ok(State::Missing);
        }
        return Err(Error::Failed {
            engine: engine.key(),
            action: "inspect",
            detail,
        });
    }
    Ok(match String::from_utf8_lossy(&stdout).trim() {
        "running" => State::Running,
        _ => State::Stopped,
    })
}

/// Start the container `id` if it is not running.
pub fn ensure_running(engine: Engine, id: &str, cancelled: &dyn Fn() -> bool) -> Result<()> {
    match state(engine, id, cancelled)? {
        State::Running => Ok(()),
        State::Missing => Err(Error::Gone(id.to_owned())),
        State::Stopped => {
            // A paused container refuses `start`; `unpause` is harmless otherwise.
            let _ = engine.output("unpause", ["unpause", id], QUICK, cancelled);
            engine.run("start", ["start", id], QUICK, cancelled)?;
            // Podman gives the volume's mount point back to root on each
            // start, which would leave the agents' user without its home.
            prepare(engine, id, cancelled)
        }
    }
}

/// A command that runs, as the `herdr` user in its home, the one shell string
/// appended to it, as `ssh` runs its remote command. The saved connector and
/// retained cloud command route use it.
pub fn exec(engine: Engine, id: &str) -> Result<Command> {
    engine.command([
        "exec",
        "--interactive",
        "--user",
        USER,
        "--workdir",
        HOME,
        "--env",
        &format!("HOME={HOME}"),
        "--env",
        &format!("USER={USER}"),
        id,
        "/bin/sh",
        "-c",
    ])
}
