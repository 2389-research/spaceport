// ABOUTME: Exercises forward ownership across abrupt parent exit.
// ABOUTME: Uses isolated processes and real loopback listeners without SSH credentials.
use super::*;
use std::net::TcpStream;

const FIXTURE: &str = "SPACEPORT_FORWARD_CRASH_FIXTURE";
const TEMP_SOCKET: &str = "SPACEPORT_FORWARD_TEMP_SOCKET";

fn temporary_socket() -> bool {
    std::env::var(TEMP_SOCKET).is_ok_and(|value| value == "yes")
}

struct Process(std::process::Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn fixture_command(name: &str, dir: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            &format!("forward::tests::crash_cleanup::{name}"),
            "--ignored",
            "--nocapture",
        ])
        .env(FIXTURE, dir);
    command
}

#[test]
#[ignore = "owned fixture subprocess"]
fn listener_process() {
    let dir = PathBuf::from(std::env::var_os(FIXTURE).unwrap());
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let socket = PathBuf::from(std::env::var_os("SPACEPORT_FORWARD_SOCKET").unwrap());
    // OpenSSH first binds c.<random> before renaming its control socket.
    let bound = if temporary_socket() {
        socket.with_extension("0123456789abcdef")
    } else {
        socket.clone()
    };
    std::fs::write(bound, []).unwrap();
    std::fs::write(dir.join("socket"), format!("{}\n", socket.display())).unwrap();
    std::fs::write(
        dir.join("listener"),
        format!("{}\n", listener.local_addr().unwrap().port()),
    )
    .unwrap();
    std::fs::write(dir.join("pid"), format!("{}\n", std::process::id())).unwrap();
    loop {
        thread::park();
    }
}

fn listener(dir: &Path) -> impl FnOnce(&Path) -> Command + Send + 'static {
    let dir = dir.to_owned();
    move |socket| {
        let mut command = fixture_command("listener_process", &dir);
        command.env("SPACEPORT_FORWARD_SOCKET", socket);
        command
    }
}

#[test]
#[ignore = "owned fixture subprocess"]
fn owner_process() {
    let dir = PathBuf::from(std::env::var_os(FIXTURE).unwrap());
    let forward =
        PortForward::start_with(port(3000), WAIT, listener(&dir), grant(&dir, 0)).unwrap();
    if temporary_socket() {
        read(&dir.join("pid"));
    } else {
        assert!(matches!(
            next_event(&forward),
            ForwardEvent::Listening { .. }
        ));
    }
    std::fs::write(dir.join("ready"), "ready\n").unwrap();
    loop {
        thread::park();
    }
}

// Keeps a failed regression run from leaving its fixture listener behind.
struct Cleanup(PathBuf, bool);
impl Drop for Cleanup {
    fn drop(&mut self) {
        if self.1
            && let Ok(pid) = std::fs::read_to_string(self.0.join("pid"))
        {
            let _ = Command::new("/bin/kill")
                .args(["-9", pid.trim()])
                .stderr(Stdio::null())
                .status();
        }
        if let Ok(socket) = std::fs::read_to_string(self.0.join("socket")) {
            let _ = std::fs::remove_dir_all(Path::new(socket.trim()).parent().unwrap());
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn forced_parent_exit_closes_the_listener_and_removes_the_control_directory() {
    for temporary in [false, true] {
        parent_cleanup(temporary);
    }
}

fn parent_cleanup(temporary: bool) {
    let dir = scratch(if temporary {
        "parent-crash-temp"
    } else {
        "parent-crash"
    });
    let mut cleanup = Cleanup(dir.clone(), true);
    let mut unrelated = Process(Command::new("sleep").arg("30").spawn().unwrap());
    let mut owner = Process(
        fixture_command("owner_process", &dir)
            .env(TEMP_SOCKET, if temporary { "yes" } else { "no" })
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    read(&dir.join("ready"));
    let pid = read(&dir.join("pid"));
    let local = read(&dir.join("listener")).trim().parse::<u16>().unwrap();
    let socket = read(&dir.join("socket"));
    let control = Path::new(socket.trim()).parent().unwrap();
    assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_ok());
    owner.0.kill().unwrap();
    owner.0.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while running(&pid) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    let stopped = !running(&pid);
    let listener_closed = TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_err();
    let unrelated_alive = unrelated.0.try_wait().unwrap().is_none();
    assert!(
        stopped,
        "owned listener process survived forced parent exit"
    );
    cleanup.1 = false;
    assert!(
        listener_closed,
        "owned loopback listener survived forced parent exit"
    );
    wait_until(
        || !control.exists(),
        "control directory survived forced parent exit",
    );
    assert!(unrelated_alive, "cleanup signalled an unrelated child");
}

#[test]
fn stopping_and_dropping_forwards_release_their_real_listeners() {
    for stop_first in [false, true] {
        let dir = scratch(if stop_first {
            "listener-stop"
        } else {
            "listener-drop"
        });
        let mut cleanup = Cleanup(dir.clone(), true);
        let forward =
            PortForward::start_with(port(3000), WAIT, listener(&dir), grant(&dir, 0)).unwrap();
        assert!(matches!(
            next_event(&forward),
            ForwardEvent::Listening { .. }
        ));
        let pid = read(&dir.join("pid"));
        let local = read(&dir.join("listener")).trim().parse::<u16>().unwrap();
        let socket = read(&dir.join("socket"));
        assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_ok());
        if stop_first {
            forward.stop();
            forward.stop();
        }
        drop(forward);
        wait_until(
            || !running(&pid),
            "forward retained its real listener child",
        );
        cleanup.1 = false;
        wait_until(
            || !Path::new(socket.trim()).parent().unwrap().exists(),
            "control directory remained after normal stop",
        );
        assert!(TcpStream::connect((Ipv4Addr::LOCALHOST, local)).is_err());
    }
}

#[test]
fn stopping_during_spawn_removes_the_private_control_directory() {
    let dir = scratch("stop-during-spawn");
    let _cleanup = Cleanup(dir.clone(), true);
    let (started, spawning) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let make_listener = listener(&dir);
    let forward = PortForward::start_with(
        port(3000),
        WAIT,
        move |socket| {
            started.send(socket.parent().unwrap().to_owned()).unwrap();
            released.recv().unwrap();
            make_listener(socket)
        },
        grant(&dir, 0),
    )
    .unwrap();
    let control = spawning.recv_timeout(WAIT).unwrap();
    forward.stop();
    release.send(()).unwrap();
    wait_until(
        || !control.exists(),
        "cancelled startup retained its control directory",
    );
    assert!(forward.try_event().is_none());
}

#[test]
fn private_forwards_override_configured_background_authentication() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("ssh_config");
    std::fs::write(
        &config,
        "Host *\n    ForkAfterAuthentication yes\n    CanonicalizeHostname no\n",
    )
    .unwrap();
    for command in [
        master_command("foreground-only.invalid", &directory.path().join("c")),
        crate::forward_command("foreground-only.invalid", 41010, None, 3000).unwrap(),
    ] {
        let output = Command::new(command.get_program())
            .env_clear()
            .env("HOME", directory.path())
            .args(["-G", "-F"])
            .arg(&config)
            .args(command.get_args())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "ssh config parser failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let options = String::from_utf8(output.stdout).unwrap();
        assert!(
            options
                .lines()
                .any(|line| line == "forkafterauthentication no")
        );
    }
}
