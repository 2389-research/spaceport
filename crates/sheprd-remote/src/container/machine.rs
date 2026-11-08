//! One container's lifecycle on the engine. A container is named by its
//! device ID, keeps its home directory in a named volume beside it, and is
//! labelled so containers this app made can be told from the user's own.
//!
//! Hardening is the engine's default seccomp and capability set plus
//! `no-new-privileges`, an init process, and CPU, memory, and process limits;
//! the container never gets the engine's socket, host networking, or a mount
//! of this computer's files. Agents work as an unprivileged `herdr` user.

use super::{Engine, Error, Result, engine::detail};
use std::{process::Command, time::Duration};

/// The label every container and volume this app makes carries, with the
/// device ID as its value.
pub const LABEL: &str = "dev.herdr-gpui.device";
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

pub fn volume(id: &str) -> String {
    format!("{id}-home")
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

/// Stop the container `id`, keeping it and its files.
pub fn stop(engine: Engine, id: &str, cancelled: &dyn Fn() -> bool) -> Result<()> {
    engine
        .run("stop", ["stop", id], Duration::from_secs(60), cancelled)
        .map(drop)
}

/// A container or a volume, for [`owner`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Container,
    Volume,
}

/// Who holds a name on the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Missing,
    /// Labelled as the device of the same ID: this app made it for that device.
    Ours,
    /// Anything else, such as the user's own container of that name.
    Other,
}

/// Who holds the container `id`, or its home volume.
fn owner(engine: Engine, kind: Kind, id: &str, cancelled: &dyn Fn() -> bool) -> Result<Owner> {
    let (name, format) = match kind {
        Kind::Container => (
            id.to_owned(),
            format!("{{{{index .Config.Labels \"{LABEL}\"}}}}"),
        ),
        Kind::Volume => (volume(id), format!("{{{{index .Labels \"{LABEL}\"}}}}")),
    };
    let noun = match kind {
        Kind::Container => "container",
        Kind::Volume => "volume",
    };
    let (code, stdout, stderr) = engine.output(
        "inspect",
        [noun, "inspect", "--format", &format, &name],
        QUICK,
        cancelled,
    )?;
    if code != Some(0) {
        let detail = detail(&stderr);
        // Docker says "No such container/volume", Podman "no such ...".
        if detail.to_ascii_lowercase().contains("no such") {
            return Ok(Owner::Missing);
        }
        return Err(Error::Failed {
            engine: engine.key(),
            action: "inspect",
            detail,
        });
    }
    Ok(if String::from_utf8_lossy(&stdout).trim() == id {
        Owner::Ours
    } else {
        Owner::Other
    })
}

/// Remove the container `id` and its home volume, when they are this app's
/// for that device. Either one already gone is not an error; one of the same
/// name that this app did not make is refused and left alone.
pub fn delete(engine: Engine, id: &str, cancelled: &dyn Fn() -> bool) -> Result<()> {
    for kind in [Kind::Container, Kind::Volume] {
        match owner(engine, kind, id, cancelled)? {
            Owner::Missing => continue,
            Owner::Other => return Err(Error::NotOurs(id.to_owned())),
            Owner::Ours => {}
        }
        let (action, args) = match kind {
            Kind::Container => (
                "rm",
                vec!["rm".to_owned(), "--force".to_owned(), id.to_owned()],
            ),
            Kind::Volume => (
                "volume rm",
                vec!["volume".to_owned(), "rm".to_owned(), volume(id)],
            ),
        };
        let (code, _, stderr) = engine.output(action, args, QUICK, cancelled)?;
        let detail = detail(&stderr);
        if code != Some(0) && !detail.to_ascii_lowercase().contains("no such") {
            return Err(Error::Failed {
                engine: engine.key(),
                action,
                detail,
            });
        }
    }
    Ok(())
}

/// A command that runs, as the `herdr` user in its home, the one shell string
/// appended to it, as `ssh` runs its remote command: the bridge, Herdr's
/// installer, and Teleport's scripts all append theirs.
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
