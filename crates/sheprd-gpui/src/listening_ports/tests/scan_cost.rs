// ABOUTME: Measures scanner process counts using private executable producers.
// ABOUTME: Keeps scan and shared-consumer fixtures outside personal process environments.
use super::*;
use crate::test_sandbox::Sandbox;
use gpui::TestAppContext;
use std::{
    fs,
    path::Path,
    process::Child,
    thread,
    time::{Duration, Instant},
};

const ROOT: &str = "SPACEPORT_SCAN_FIXTURE_ROOT";
const WAIT: Duration = Duration::from_secs(20);

fn executable(root: &Path, name: &str, body: &str) {
    crate::test_executable::write(
        root.join(".local/bin").join(name),
        format!("#!/bin/sh\n{body}\n"),
        0o700,
    )
    .unwrap();
}

pub(super) fn owned_environment_and_listener() {
    use std::process::Stdio;
    let root = Sandbox::new();
    fs::create_dir_all(root.dir.join(".local/bin")).unwrap();
    let socket = root.dir.join("socket with spaces/herdr.sock");
    let mut command = root.command(std::env::current_exe().unwrap(), "listener.log");
    command
        .env("SPACEPORT_TUNNEL_CRASH_FIXTURE", &root.dir)
        .env("HERDR_WORKSPACE_ID", "owned-workspace")
        .env("HERDR_SOCKET_PATH", &socket)
        .args([
            "--exact",
            "listening_ports::tunnel::tests::listener_process",
            "--ignored",
            "--nocapture",
        ]);
    let mut owned = Process(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(5);
    while !fs::read_to_string(root.dir.join("port")).is_ok_and(|port| port.ends_with('\n')) {
        assert!(
            owned.0.try_wait().unwrap().is_none(),
            "owned listener exited before readiness:\n{}",
            fs::read_to_string(root.dir.join("listener.log")).unwrap()
        );
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    let pid = owned.0.id();
    let output = root
        .command("/bin/ps", "ps.log")
        .args([
            "eww",
            "-o",
            "pid=",
            "-o",
            "command=",
            "-p",
            &pid.to_string(),
        ])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(output.status.success(), "owned ps query failed");
    assert_eq!(text.lines().count(), 1, "ps selects only our exact child");
    assert_eq!(
        text.split_whitespace().next(),
        Some(pid.to_string().as_str())
    );
    assert!(
        text.contains("HERDR_WORKSPACE_ID=owned-workspace"),
        "{text}"
    );
    assert!(
        text.contains(&format!("HERDR_SOCKET_PATH={}", socket.display())),
        "{text}"
    );
    let port = fs::read_to_string(root.dir.join("port")).unwrap();
    let tool = root
        .command("/bin/sh", "tool.log")
        .args(["-c", "command -v ss || command -v lsof"])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    if tool.status.success() {
        let tool = String::from_utf8(tool.stdout).unwrap();
        let tool = tool.trim();
        if tool.ends_with("/ss") {
            executable(
                &root.dir,
                "ss",
                &format!("'{tool}' \"$@\" | awk 'index($0, \"pid={pid},\") {{print}}'"),
            );
        } else {
            executable(
                &root.dir,
                "lsof",
                &format!("exec '{tool}' -a -p {pid} \"$@\""),
            );
        }
    }
    let mut command = root.command("/bin/sh", "scan.log");
    command
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    let mut shell = crate::usage::Shell::start(command).unwrap();
    let output = shell
        .run(super::super::scan::COMMAND, super::super::STEP_TIMEOUT)
        .unwrap();
    if !tool.status.success() {
        assert!(matches!(
            parse(&output.stdout),
            Err(Error::ListeningPorts(super::super::Error::Tool))
        ));
        return;
    }
    let expected = format!("L {pid} 127.0.0.1:{} ", port.trim());
    assert!(
        output
            .stdout
            .lines()
            .any(|line| line.starts_with(&expected)),
        "{}",
        output.stdout
    );
    let ports = parse(&output.stdout).unwrap();
    let owner = ports.keys().next().unwrap();
    assert_eq!(owner.workspace, "owned-workspace");
    assert_eq!(owner.socket, socket.to_str().unwrap());
    assert_eq!(
        ports.values().next().unwrap()[0].number.to_string(),
        port.trim()
    );
}

fn fixture(listeners: usize) -> Sandbox {
    let root = Sandbox::new();
    fs::create_dir_all(root.dir.join(".local/bin")).unwrap();
    executable(
        &root.dir,
        "ss",
        &format!(
            r#"printf 'scan\n' >> "$SPACEPORT_SCAN_FIXTURE_ROOT/scans"
if [ -e "$SPACEPORT_SCAN_FIXTURE_ROOT/break-shell" ]; then
    kill -KILL "$(cat "$SPACEPORT_SCAN_FIXTURE_ROOT/shell.pid")"
    exit
fi
base=3000
if [ -f "$SPACEPORT_SCAN_FIXTURE_ROOT/base" ]; then base=$(cat "$SPACEPORT_SCAN_FIXTURE_ROOT/base"); fi
i=0
while [ "$i" -lt {listeners} ]; do
    printf 'LISTEN 0 128 127.0.0.1:%s 0.0.0.0:* users:(("fixture",pid=%s,fd=3))\n' "$((base+i))" "$((2147480000+i))"
    i=$((i+1))
done"#
        ),
    );
    executable(
        &root.dir,
        "ps",
        r#"printf 'ps\n' >> "$SPACEPORT_SCAN_FIXTURE_ROOT/ps-calls"
ids=
want_ids=false
previous=
for argument do
    if [ "$previous" = -p ]; then ids=$argument; fi
    if [ "$argument" = pid= ]; then want_ids=true; fi
    previous=$argument
done
for pid in $(printf '%s' "$ids" | tr ',' ' '); do
    if $want_ids; then printf '%s ' "$pid"; fi
    printf 'fixture HERDR_WORKSPACE_ID=w%s HERDR_SOCKET_PATH=%s/herdr.sock NEXT=done\n' "$((pid-2147480000))" "$SPACEPORT_SCAN_FIXTURE_ROOT"
done"#,
    );
    executable(
        &root.dir,
        "ssh",
        r#"for argument do last=$argument; done
exec /bin/sh -c "$last""#,
    );
    root
}

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub(in crate::listening_ports) fn run(name: &str, listeners: usize) {
    let root = fixture(listeners);
    let mut command = root.command(std::env::current_exe().unwrap(), "scan.log");
    let name = if name.contains("::") {
        name.to_owned()
    } else {
        format!("listening_ports::tests::scan_cost::{name}")
    };
    command
        .env(ROOT, &root.dir)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.dir.join(".local/bin").display()),
        )
        .args(["--exact", &name, "--ignored", "--nocapture"]);
    let mut child = Process(command.spawn().unwrap());
    let deadline = Instant::now() + WAIT;
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "private scanner fixture timed out"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "private scanner fixture failed: {}",
        fs::read_to_string(root.dir.join("scan.log")).unwrap()
    );
}

pub(in crate::listening_ports) fn private_root() -> std::path::PathBuf {
    let root = std::path::PathBuf::from(std::env::var_os(ROOT).unwrap());
    assert_eq!(
        std::env::var_os("HOME").map(std::path::PathBuf::from),
        Some(root.clone())
    );
    root
}

#[test]
fn environment_lookup_batches_the_bounded_pid_set() {
    run("batch_fixture", 128);
}

#[test]
#[ignore = "private scanner producer, executed by its nonignored parent"]
fn batch_fixture() {
    let root = private_root();
    let mut shell = super::super::local_shell().unwrap();
    let output = shell
        .run(super::super::scan::COMMAND, super::super::STEP_TIMEOUT)
        .unwrap();
    let ports = parse(&output.stdout).unwrap();
    assert_eq!(
        ports.len(),
        128,
        "all private listener owners survive batching"
    );
    let calls = fs::read_to_string(root.join("ps-calls")).unwrap();
    assert_eq!(
        calls.lines().count(),
        1,
        "one bounded ps lookup replaces the per-PID pipelines"
    );
}

#[test]
fn matching_consumers_share_one_host_scan() {
    run("shared_fixture", 1);
}

#[gpui::test]
#[ignore = "private scanner consumers, executed by their nonignored parent"]
fn shared_fixture(cx: &mut TestAppContext) {
    let root = private_root();
    let mut first = ListeningPorts::default();
    let mut second = ListeningPorts::default();
    let daemon = Daemon::of(&ConnectTarget::Socket(root.join("herdr-client.sock"))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        cx.update(|cx| {
            first.poll([Host::Local], cx);
            second.poll([Host::Local], cx);
        });
        if first.get(&daemon, "w0").is_some() && second.get(&daemon, "w0").is_some() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "both consumers must receive the private producer's ports"
        );
        thread::sleep(Duration::from_millis(5));
    }
    let scans = fs::read_to_string(root.join("scans")).unwrap();
    assert_eq!(
        scans.lines().count(),
        1,
        "matching consumers lease one actual host worker"
    );
}
