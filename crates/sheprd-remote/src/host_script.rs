//! ABOUTME: Runs bounded scripts against an explicitly selected Herdr host.
//! ABOUTME: Preserves session targeting, cancellation, and the shared SSH query lane.

mod error;

pub use error::{Error, Result};
use sheprd_client::{ConnectTarget, ScriptHost, ScriptLimits, SshLane, run_script, shell_quote};
use std::{
    io::{Read, Write},
    sync::atomic::AtomicBool,
    time::Duration,
};

/// Default output bound for host queries.
const QUERY_OUTPUT: u64 = 32 * 1024 * 1024;
const QUERY_IDLE: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    Local,
    Ssh(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    route: Route,
    session: Option<String>,
    /// The session belongs to the development installation, which
    /// `herdr --session NAME` cannot address: NAME would resolve to the
    /// release session that shares it. Ordinary scripts still run, but
    /// `herdr_cli` refuses.
    development: bool,
    /// Tests run "remote" hosts locally in their own environment (home,
    /// daemon socket), exported at the start of every script.
    #[cfg(any(test, feature = "test-support"))]
    pub env: Vec<(String, String)>,
}

impl Host {
    /// How long a query may go without output.
    pub const QUERY_IDLE: Duration = QUERY_IDLE;
    /// Most a query may print.
    pub const QUERY_OUTPUT: u64 = QUERY_OUTPUT;

    /// A host for `target`. Custom sockets are refused: the CLI could not be
    /// pointed at them without also trusting that socket's daemon identity.
    pub fn new(target: &ConnectTarget) -> Result<Self> {
        match target {
            ConnectTarget::Local => Ok(Self {
                route: Route::Local,
                session: None,
                development: false,
                #[cfg(any(test, feature = "test-support"))]
                env: Vec::new(),
            }),
            ConnectTarget::Session { name, development } => Ok(Self {
                route: Route::Local,
                session: Some(name.clone()),
                development: *development,
                #[cfg(any(test, feature = "test-support"))]
                env: Vec::new(),
            }),
            ConnectTarget::Ssh { target, session } => Ok(Self {
                route: Route::Ssh(target.clone()),
                session: Some(session.clone()),
                development: false,
                #[cfg(any(test, feature = "test-support"))]
                env: Vec::new(),
            }),
            ConnectTarget::Socket(_) | ConnectTarget::Wsl { .. } => Err(Error::UnsupportedHost),
        }
    }

    /// Whether `herdr_cli` can reach this host's session; see `development`.
    pub fn scripts_herdr(&self) -> bool {
        !self.development
    }

    /// The prelude every script starts with: strict mode, a `PATH` that finds
    /// user-installed tools in a non-interactive SSH shell, and `herdr_cli`
    /// bound to this host's session, or refusing when it cannot be reached.
    fn prelude(&self) -> String {
        let session = self
            .session
            .as_deref()
            .map(|session| format!(" --session {}", shell_quote(session)))
            .unwrap_or_default();
        #[cfg(any(test, feature = "test-support"))]
        let home = self
            .env
            .iter()
            .map(|(key, value)| format!("{key}={}\nexport {key}\n", shell_quote(value)))
            .collect::<String>();
        #[cfg(not(any(test, feature = "test-support")))]
        let home = "";
        let herdr_cli = if self.development {
            r#"herdr_cli() { echo "herdr: a development (--dev) session cannot be scripted" >&2; return 97; }"#
                .to_owned()
        } else {
            format!(r#"herdr_cli() {{ herdr{session} "$@"; }}"#)
        };
        format!(
            r#"set -eu
{home}herdr_linuxbrew=
case "$(uname -s)" in Linux) herdr_linuxbrew=/home/linuxbrew/.linuxbrew/bin:;; esac
PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$HOME/.bun/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$herdr_linuxbrew$HOME/.nix-profile/bin:$PATH"
export PATH
{herdr_cli}
"#
        )
    }

    /// Run a small script with bounded input/output on the shared query lane.
    /// Output already collected remains available when the script fails.
    pub fn stream(
        &self,
        body: &str,
        input: impl Read + Send,
        output: impl Write + Send,
        limits: ScriptLimits,
        cancelled: &AtomicBool,
    ) -> sheprd_client::Result<u64> {
        self.stream_on(SshLane::Quick, body, input, output, limits, cancelled)
    }

    fn stream_on(
        &self,
        lane: SshLane,
        body: &str,
        input: impl Read + Send,
        output: impl Write + Send,
        limits: ScriptLimits,
        cancelled: &AtomicBool,
    ) -> sheprd_client::Result<u64> {
        let script_text = format!("{}{body}", self.prelude());
        let host = match &self.route {
            Route::Local => ScriptHost::Local,
            Route::Ssh(target) => ScriptHost::Ssh(target, lane),
        };
        run_script(host, &script_text, input, output, limits, cancelled)
    }

    /// Run `body` with no input and collect its bounded stdout.
    pub fn capture(&self, body: &str, cancelled: &AtomicBool) -> sheprd_client::Result<Vec<u8>> {
        self.capture_within(body, QUERY_IDLE, cancelled)
    }

    /// [`Host::capture`] for callers on a deadline shorter than the default
    /// query idle timeout.
    pub fn capture_within(
        &self,
        body: &str,
        idle: Duration,
        cancelled: &AtomicBool,
    ) -> sheprd_client::Result<Vec<u8>> {
        let mut output = Vec::new();
        self.stream(
            body,
            &[][..],
            &mut output,
            ScriptLimits {
                output: QUERY_OUTPUT,
                idle: idle.min(QUERY_IDLE),
            },
            cancelled,
        )?;
        Ok(output)
    }

    /// The script line running `herdr_cli` with `args`, each quoted.
    pub fn herdr_line(args: &[&str]) -> String {
        let command = args
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" ");
        format!("herdr_cli {command}\n")
    }
}

/// Select a host only when its `herdr_cli` can address the requested session.
pub fn host_for(target: &ConnectTarget) -> Result<Host> {
    let host = Host::new(target)?;
    if !host.scripts_herdr() {
        return Err(Error::DevelopmentSession);
    }
    Ok(host)
}

#[cfg(test)]
mod tests;
