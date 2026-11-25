//! Edits to the saved device catalog, delegated to the installed Herdr CLI:
//! removing and renaming a saved device, and the label a device is saved with.
mod error;

pub use error::{Error, Result};
use spaceport_client::Destination;
use std::{
    ffi::OsStr,
    io::Read,
    process::{Command, ExitStatus, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};

/// Enough for the CLI's final error line without retaining its output.
const STDERR_LIMIT: u64 = 8 * 1024;
/// `machine remove` and `machine rename` edit one local file.
const REMOVE_TIMEOUT: Duration = Duration::from_secs(15);

/// Hosts whose saved profile some window of this process is removing, so two
/// windows, or two clicks, cannot remove the same host at once.
static CLAIMS: Mutex<Vec<(Destination, String)>> = Mutex::new(Vec::new());

/// This process is removing one host and session. Dropping it releases the
/// host.
#[derive(Debug)]
pub struct Claim {
    destination: Destination,
    session: String,
}

impl Claim {
    fn acquire(destination: Destination, session: &str) -> Result<Self> {
        let mut claims = CLAIMS.lock().unwrap_or_else(|error| error.into_inner());
        if claims
            .iter()
            .any(|(claimed, claimed_session)| *claimed == destination && claimed_session == session)
        {
            tracing::warn!(
                category = "device_claim",
                claims = claims.len(),
                "Host is already claimed by this process"
            );
            return Err(Error::Busy);
        }
        tracing::debug!(category = "device_claim", "Host claimed");
        claims.push((destination.clone(), session.to_owned()));
        Ok(Self {
            destination,
            session: session.to_owned(),
        })
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        let mut claims = CLAIMS.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(index) = claims
            .iter()
            .position(|(claimed, session)| *claimed == self.destination && *session == self.session)
        {
            claims.remove(index);
            tracing::debug!(category = "device_claim", "Host released");
        }
    }
}

/// Claim a saved host while it is removed, so a second removal of the same
/// host from this process cannot run at the same time. Blocks on `ssh -G`.
pub fn claim_saved(target: &str, session: &str) -> Result<Claim> {
    let destination = spaceport_client::resolve_destination(target)?;
    Claim::acquire(destination, session)
}

/// Forget a saved device with `herdr machine remove`, which only edits the
/// local catalog: it never connects, so the host's own Herdr keeps running.
/// `executable` is the installed CLI. Blocks on a process: call it from the
/// background executor.
pub fn remove(executable: &OsStr, id: &str) -> Result<()> {
    if !spaceport_client::valid_profile_id(id) {
        tracing::warn!(category = "device_remove", "Invalid profile ID");
        return Err(Error::NoProfile);
    }
    let (status, _, stderr) = run_cli(executable, &["machine", "remove", id], REMOVE_TIMEOUT)?;
    if status.success() {
        return Ok(());
    }
    Err(Error::Remove {
        status,
        detail: last_line(&stderr),
    })
}

/// Rename a saved device with `herdr machine rename`, which edits only the
/// local catalog. Blocks on a process: call it from the background executor.
pub fn rename(executable: &OsStr, id: &str, label: &str) -> Result<()> {
    if !spaceport_client::valid_profile_id(id) {
        return Err(Error::NoProfile);
    }
    // The `=` form keeps a label that starts with `-` from reading as a flag.
    let (status, _, stderr) = run_cli(
        executable,
        &["machine", "rename", id, &format!("--label={label}")],
        REMOVE_TIMEOUT,
    )?;
    if status.success() {
        return Ok(());
    }
    Err(Error::Rename {
        status,
        detail: last_line(&stderr),
    })
}

/// Herdr caps device labels at this many bytes.
const LABEL_LIMIT: usize = 128;

/// The label a device is saved or renamed with. An empty one names the device
/// after its SSH target as typed, such as `user@host` or an address.
pub fn device_label(label: &str, target: &str) -> Result<String> {
    let label = label.trim();
    let label = if label.is_empty() {
        let target = target.trim();
        let target = target.strip_prefix("ssh://").unwrap_or(target);
        // A long target is shortened on a character boundary to fit.
        let mut end = target.len().min(LABEL_LIMIT);
        while !target.is_char_boundary(end) {
            end -= 1;
        }
        &target[..end]
    } else {
        label
    };
    if label.is_empty() || label.len() > LABEL_LIMIT || label.chars().any(char::is_control) {
        return Err(Error::InvalidLabel);
    }
    Ok(label.to_owned())
}

/// The variables that choose where Herdr keeps its device catalog. Every CLI
/// run for a device gets this process's values, so it edits the same catalog.
const CATALOG_ROOTS: [&str; 3] = ["HERDR_CONFIG_PATH", "XDG_STATE_HOME", "XDG_CONFIG_HOME"];

/// The environment of every Herdr CLI run for a device: the user's login
/// environment, not launchd's, and this window's catalog. Blocks the first time the
/// login environment is read: call it off the UI thread.
fn cli_environment(command: &mut Command) {
    spaceport_client::login_env::apply(command);
    for key in CATALOG_ROOTS {
        match std::env::var_os(key) {
            Some(value) => command.env(key, value),
            None => command.env_remove(key),
        };
    }
}

/// Runs the CLI with closed stdin and a deadline, keeping bounded stdout and
/// stderr.
fn run_cli(
    executable: &OsStr,
    args: &[&str],
    timeout: Duration,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    run_bounded(executable, args, timeout, STDERR_LIMIT)
}

/// Runs a program with closed stdin and a deadline, keeping at most `limit`
/// bytes of each of stdout and stderr. Both pipes are drained so a chatty
/// program never blocks on a full one. A missed deadline kills the program and
/// returns `Error::SetupTimeout`. Blocks, also on the first login-environment
/// lookup: call it from the background executor.
fn run_bounded(
    executable: &OsStr,
    args: &[&str],
    timeout: Duration,
    limit: u64,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let started = Instant::now();
    let result = run_bounded_untraced(executable, args, timeout, limit);
    // Only the subcommand: later arguments name hosts and labels, and the
    // output can carry remote text, so neither is logged.
    let command = &args[..args.len().min(2)];
    let elapsed_ms = started.elapsed().as_millis() as u64;
    match &result {
        Ok((status, ..)) => tracing::info!(
            category = "device_cli",
            ?command,
            %status,
            elapsed_ms,
            "Device command finished"
        ),
        Err(_) => tracing::warn!(
            category = "device_cli",
            ?command,
            elapsed_ms,
            "Device command failed to run"
        ),
    }
    result
}

fn run_bounded_untraced(
    executable: &OsStr,
    args: &[&str],
    timeout: Duration,
    limit: u64,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let mut command = spaceport_client::new_command(executable);
    cli_environment(&mut command);
    let mut child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::Builder::new()
            .name("herdr-device-cli".into())
            .spawn(move || {
                let mut output = Vec::new();
                if let Some(mut pipe) = pipe {
                    let _ = (&mut pipe).take(limit).read_to_end(&mut output);
                    let _ = std::io::copy(&mut pipe, &mut std::io::sink());
                }
                output
            })
    };
    let stdout = drain(
        child
            .stdout
            .take()
            .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
    )?;
    let stderr = drain(
        child
            .stderr
            .take()
            .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
    )?;
    let status = match spaceport_client::wait::exit(&mut child, Instant::now() + timeout, || false)?
    {
        spaceport_client::wait::Exit::Exited(status) => status,
        spaceport_client::wait::Exit::Stopped(_) => return Err(Error::SetupTimeout),
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = stderr.join().unwrap_or_default();
    Ok((status, stdout, stderr))
}

/// The CLI ends with its most specific error; earlier lines are progress.
fn last_line(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .take(300)
        .collect()
}

#[cfg(test)]
mod tests;
