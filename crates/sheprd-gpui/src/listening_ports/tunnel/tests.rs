// ABOUTME: Exercises click-tunnel cleanup with real process and socket ownership.
// ABOUTME: Covers forced parent exit, drop, and cancellation without remote credentials.
use super::*;
use std::{
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
    process::Command,
};

const FIXTURE: &str = "SPACEPORT_TUNNEL_CRASH_FIXTURE";
const WAIT: Duration = Duration::from_secs(5);

struct Process(std::process::Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn command(name: &str, directory: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            &format!("listening_ports::tunnel::tests::{name}"),
            "--ignored",
            "--nocapture",
        ])
        .env(FIXTURE, directory);
    command
}

fn wait_until(mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + WAIT;
    while !done() {
        assert!(
            Instant::now() < deadline,
            "owned tunnel did not release its listener"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn port(directory: &Path) -> u16 {
    wait_until(|| std::fs::read_to_string(directory.join("port")).is_ok_and(|s| s.ends_with('\n')));
    std::fs::read_to_string(directory.join("port"))
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

#[test]
#[ignore = "owned fixture subprocess"]
fn listener_process() {
    let directory = PathBuf::from(std::env::var_os(FIXTURE).unwrap());
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    std::fs::write(directory.join("pid"), format!("{}", std::process::id())).unwrap();
    std::fs::write(
        directory.join("port"),
        format!("{}\n", listener.local_addr().unwrap().port()),
    )
    .unwrap();
    if std::env::var_os("SPACEPORT_TUNNEL_NO_READY").is_none() {
        println!("{}", sheprd_client::FORWARD_READY);
        std::io::stdout().flush().unwrap();
    }
    loop {
        thread::park();
    }
}

#[test]
#[ignore = "owned fixture subprocess"]
fn owner_process() {
    let directory = PathBuf::from(std::env::var_os(FIXTURE).unwrap());
    let attempt = Attempt::default();
    let _tunnel = start_command(41010, command("listener_process", &directory), &attempt).unwrap();
    std::fs::write(directory.join("ready"), []).unwrap();
    loop {
        thread::park();
    }
}

// Only used after a failed regression leaves the fixture's listener open.
struct Cleanup(PathBuf);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let Ok(pid) = std::fs::read_to_string(self.0.join("pid")) else {
            return;
        };
        let local = std::fs::read_to_string(self.0.join("port"))
            .ok()
            .and_then(|port| port.trim().parse::<u16>().ok());
        if local.is_some_and(|local| TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_err()) {
            return;
        }
        #[cfg(unix)]
        let _ = Command::new("/bin/kill")
            .args(["-9", &pid])
            .stderr(Stdio::null())
            .status();
        #[cfg(windows)]
        let _ = Command::new("taskkill.exe")
            .args(["/F", "/PID", &pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

#[test]
fn forced_parent_exit_closes_the_click_tunnel_listener() {
    let directory = tempfile::tempdir().unwrap();
    let _cleanup = Cleanup(directory.path().to_owned());
    // A separately owned listener must survive the owner's death.
    let unrelated = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let mut owner = Process(
        command("owner_process", directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    wait_until(|| directory.path().join("ready").exists());
    let local = port(directory.path());
    assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_ok());
    owner.0.kill().unwrap();
    owner.0.wait().unwrap();
    wait_until(|| TcpListener::bind((Ipv4Addr::LOCALHOST, local)).is_ok());
    assert!(TcpStream::connect(unrelated.local_addr().unwrap()).is_ok());
}

#[test]
fn dropping_and_cancelling_click_tunnels_release_their_listeners() {
    for cancel_first in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let _cleanup = Cleanup(directory.path().to_owned());
        let attempt = Attempt::default();
        let tunnel = start_command(
            41010,
            command("listener_process", directory.path()),
            &attempt,
        )
        .unwrap();
        let local = port(directory.path());
        assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_ok());
        if cancel_first {
            attempt.cancel();
            attempt.cancel();
        }
        drop(tunnel);
        wait_until(|| TcpListener::bind((Ipv4Addr::LOCALHOST, local)).is_ok());
    }
}

#[test]
fn cancellation_during_readiness_ends_the_real_listener() {
    let directory = tempfile::tempdir().unwrap();
    let _cleanup = Cleanup(directory.path().to_owned());
    let attempt = Attempt::default();
    let shared = attempt.clone();
    let mut command = command("listener_process", directory.path());
    command.env("SPACEPORT_TUNNEL_NO_READY", "1");
    let opening = thread::spawn(move || start_command(41010, command, &shared));
    let local = port(directory.path());
    attempt.cancel();
    assert!(opening.join().unwrap().is_err());
    wait_until(|| TcpListener::bind((Ipv4Addr::LOCALHOST, local)).is_ok());
}
