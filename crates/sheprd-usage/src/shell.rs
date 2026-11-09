// ABOUTME: Bounded shell steps for listening-port inspection over SSH or WSL.
// ABOUTME: Owns its child and SSH channel and refuses reuse after timeout or read failure.
use crate::{Error, Result};
use std::{
    io::{Read, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const LIMIT: usize = 1024 * 1024;

/// A bounded shell step result.
pub struct Output {
    pub success: bool,
    pub stdout: String,
}

/// Long enough for `wsl.exe` to boot a stopped distribution.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// Establish the existing executable search path for each host.
const PRELUDE: &str = r#"herdr_linuxbrew=
case "$(uname -s)" in Linux) herdr_linuxbrew=/home/linuxbrew/.linuxbrew/bin:;; esac
PATH="$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:/opt/homebrew/bin:/usr/local/bin:$herdr_linuxbrew$PATH"
export PATH
"#;

/// A `/bin/sh` on a remote host, fed one step at a time over SSH stdin. Each
/// step ends with a marker carrying its exit status, so steps can be read
/// back without closing the session.
pub struct Shell {
    child: Child,
    stdin: ChildStdin,
    output: mpsc::Receiver<std::io::Result<Vec<u8>>>,
    buffer: Vec<u8>,
    marker: String,
    broken: bool,
    /// The SSH host's shared channel this shell runs on, if any. Fields drop
    /// after `Drop::drop` reaps the child, so it is freed only then.
    channel: Option<sheprd_client::SshChannel>,
}

impl Drop for Shell {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Shell {
    pub fn connect(target: &str) -> Result<Self> {
        let (mut command, channel) = sheprd_client::shared_script_command(
            target,
            "exec /bin/sh -s",
            sheprd_client::SshLane::Quick,
        )?;
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut shell = Self::start(command)?;
        shell.channel = channel;
        Ok(shell)
    }

    /// `/bin/sh` inside a WSL distribution, through `wsl.exe`.
    pub fn connect_wsl(distro: &str) -> Result<Self> {
        if !cfg!(windows) {
            return Err(Error::WslHostUnsupported);
        }
        let mut command = sheprd_client::wsl_shell_command(distro)?;
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        Self::start(command)
    }

    /// Any `sh` reading steps from stdin: SSH or `wsl.exe` in production, a
    /// local shell in tests.
    pub fn start(mut command: Command) -> Result<Self> {
        let process = |source| Error::Process {
            operation: "start the remote usage shell",
            source,
        };
        let mut child = command.spawn().map_err(process)?;
        let (Some(stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(process(std::io::Error::other("no pipes")));
        };
        let (sender, output) = mpsc::sync_channel(64);
        let reader = thread::Builder::new()
            .name("herdr-usage-shell".into())
            .spawn(move || {
                let mut chunk = [0; 8192];
                loop {
                    match stdout.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => {
                            if sender.send(Ok(chunk[..n].to_vec())).is_err() {
                                break;
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            break;
                        }
                    }
                }
            });
        if let Err(source) = reader {
            let _ = child.kill();
            let _ = child.wait();
            return Err(process(source));
        }
        let mut shell = Self {
            child,
            stdin,
            output,
            buffer: Vec::new(),
            marker: format!("@@herdr-{}", uuid::Uuid::new_v4().simple()),
            broken: false,
            channel: None,
        };
        shell.write(PRELUDE)?;
        let ready = shell.run("true", CONNECT_TIMEOUT)?;
        if !ready.success {
            return Err(Error::Unreachable);
        }
        Ok(shell)
    }

    fn write(&mut self, text: &str) -> Result<()> {
        self.stdin
            .write_all(text.as_bytes())
            .and_then(|()| self.stdin.flush())
            .map_err(|_| {
                self.broken = true;
                Error::Unreachable
            })
    }

    /// Runs `step` and returns what it printed. A step that overruns breaks
    /// the session, since its output could still arrive later.
    pub fn run(&mut self, step: &str, timeout: Duration) -> Result<Output> {
        if self.broken {
            return Err(Error::Unreachable);
        }
        let marker = self.marker.clone();
        self.write(&format!(
            "{{ {step}\n}} </dev/null; printf '\\n{marker} %s\\n' \"$?\"\n"
        ))?;
        let deadline = Instant::now() + timeout;
        let end = format!("\n{marker} ");
        loop {
            if let Some(start) = find(&self.buffer, end.as_bytes()) {
                if start > LIMIT {
                    self.broken = true;
                    return Err(Error::Size);
                }
                let tail = start + end.len();
                if let Some(newline) = self.buffer[tail..].iter().position(|b| *b == b'\n') {
                    let status = String::from_utf8_lossy(&self.buffer[tail..tail + newline])
                        .trim()
                        .parse::<i32>()
                        .unwrap_or(1);
                    let stdout = String::from_utf8_lossy(&self.buffer[..start]).into_owned();
                    self.buffer.drain(..tail + newline + 1);
                    return Ok(Output {
                        success: status == 0,
                        stdout,
                    });
                }
            }
            if self.buffer.len() > LIMIT {
                self.broken = true;
                return Err(Error::Size);
            }
            let left = deadline.saturating_duration_since(Instant::now());
            match self.output.recv_timeout(left) {
                Ok(Ok(chunk)) => self.buffer.extend_from_slice(&chunk),
                Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.broken = true;
                    return Err(Error::Unreachable);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.broken = true;
                    return Err(Error::Timeout);
                }
            }
        }
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(all(test, unix))]
mod tests;
