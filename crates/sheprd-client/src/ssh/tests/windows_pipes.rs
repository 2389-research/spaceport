// ABOUTME: Exercises the Windows bridge over an exact owned child's anonymous pipes.
// ABOUTME: Separates bounded fixture startup from the real handshake, cancellation, and EOF contracts.
//! The compiled test child supplies only authored bridge traffic, without a shell startup dependency.
use super::*;
use std::io::BufRead as _;

const COMPATIBLE: &str = r#"{"endpoint_protocol_generation":1,"endpoint_capabilities":["surface_interest","presentation_effects_fence","health_check"],"remote_bridge_idle_timeout":true}"#;

/// Bounds native child startup and completion independently of banner parsing.
const STARTUP: Duration = Duration::from_secs(30);
const FIXTURE: &str = "HERDR_TEST_SSH_PIPE_CASE";
const READY: &[u8] = b"spaceport-owned-pipe-ready\n";

enum Case {
    // keep-sorted start
    Handshake,
    Idle,
    Oversized,
    // keep-sorted end
}

fn fixture(case: Case) -> Command {
    let mut command = crate::new_command(std::env::current_exe().unwrap());
    command
        .env_clear()
        .args([
            "--exact",
            "ssh::tests::windows_pipes::owned_pipe_fixture",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(
            FIXTURE,
            match case {
                Case::Handshake => "handshake",
                Case::Idle => "idle",
                Case::Oversized => "oversized",
            },
        );
    for name in ["SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

fn fixture_ready(stream: &mut Stream) {
    stream.set_read_timeout(Some(POLL)).unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let deadline = Instant::now() + STARTUP;
    let mut preamble = Vec::new();
    loop {
        assert!(
            Instant::now() < deadline,
            "owned pipe fixture did not become ready"
        );
        assert!(
            preamble.len() < 16_384,
            "owned pipe fixture preamble exceeds bound"
        );
        let mut byte = [0];
        match stream.read(&mut byte) {
            Ok(0) => panic!("owned pipe fixture closed during startup"),
            Ok(_) => preamble.push(byte[0]),
            Err(error) if error.kind() == io::ErrorKind::TimedOut => continue,
            Err(error) => panic!("{error}"),
        }
        // libtest can print a partial test-name line before the sentinel.
        if preamble.ends_with(READY) {
            break;
        }
    }
    stream.write_all(b"fixture-go\n").unwrap();
}

#[test]
#[ignore = "owned native pipe subprocess; its parent provides the private mode and acknowledgment"]
fn owned_pipe_fixture() {
    let case = match std::env::var(FIXTURE).unwrap().as_str() {
        "handshake" => Case::Handshake,
        "idle" => Case::Idle,
        "oversized" => Case::Oversized,
        _ => panic!("invalid owned pipe fixture case"),
    };
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    output.write_all(READY).unwrap();
    output.flush().unwrap();
    let mut line = String::new();
    input.read_line(&mut line).unwrap();
    assert_eq!(line, "fixture-go\n");
    line.clear();
    match case {
        Case::Handshake => {
            write!(
                output,
                "motd\n{COMPATIBLE}\n\nherdr-remote-output-ready:1\n"
            )
            .unwrap();
            output.flush().unwrap();
            input.read_line(&mut line).unwrap();
            assert_eq!(line, "accept-idle\n");
            line.clear();
            input.read_line(&mut line).unwrap();
            output.write_all(line.as_bytes()).unwrap();
        }
        Case::Idle => {
            output.write_all(b"herdr-remote").unwrap();
            output.flush().unwrap();
            input.read_line(&mut line).unwrap();
        }
        Case::Oversized => {
            output.write_all(&vec![b'x'; 20_000]).unwrap();
            output.flush().unwrap();
            input.read_line(&mut line).unwrap();
        }
    }
    output.flush().unwrap();
    // Exit after flushing so libtest cannot append bytes to the bridge's EOF check.
    std::process::exit(0);
}

/// Reads exactly `expected.len()` bytes, retrying the emulated timeouts an
/// idle pipe reports, and returns them.
fn read_expected(stream: &mut Stream, expected: &[u8]) -> Vec<u8> {
    let deadline = Instant::now() + STARTUP;
    let mut read = vec![0; expected.len()];
    let mut filled = 0;
    while filled < read.len() {
        assert!(Instant::now() < deadline, "timed out after {filled} bytes");
        match stream.read(&mut read[filled..]) {
            Ok(0) => panic!("closed after {filled} bytes"),
            Ok(n) => filled += n,
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {}
            Err(error) => panic!("{error}"),
        }
    }
    read
}

#[test]
fn the_handshake_and_the_bridged_stream_run_over_the_childs_pipes() {
    let stop = AtomicBool::new(false);
    let Ok((mut stream, _child)) = start_bridge(fixture(Case::Handshake), &stop, |stream, stop| {
        fixture_ready(stream);
        read_expected(stream, b"m");
        handshake(stream, stop)
    })
    .unwrap() else {
        panic!("the bridge closed before it was ready");
    };
    stream.write_all(b"hello\n").unwrap();
    assert_eq!(read_expected(&mut stream, b"hello\n"), b"hello\n");
    // Only the child held the pipes' other ends, so its exit reads as the end
    // of the stream rather than as one more idle timeout.
    let deadline = Instant::now() + STARTUP;
    let mut byte = [0];
    loop {
        assert!(Instant::now() < deadline, "the exit never read as closed");
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => panic!("unexpected output"),
            Err(error) if error.kind() == io::ErrorKind::TimedOut => {}
            Err(error) => panic!("{error}"),
        }
    }
}

#[test]
fn an_ssh_failure_before_the_banner_is_classified_from_its_stderr() {
    let mut command = crate::new_command("cmd.exe");
    command.args([
        "/d",
        "/c",
        "echo Host key verification failed.1>&2& exit 255",
    ]);
    let started = Instant::now();
    let Err(closed) = start_bridge(command, &AtomicBool::new(false), handshake).unwrap() else {
        panic!("a failed ssh was accepted");
    };
    assert_eq!(closed.code, Some(255));
    assert_eq!(closed.failure, SshFailure::HostKey);
    // Reported when the pipe closed, well before the banner's deadline.
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn an_idle_pipe_times_out_rather_than_closing_and_cancellation_ends_the_wait() {
    // Half a ready line, then nothing until stdin closes.
    let (mut stream, child) = spawn_bridge(fixture(Case::Idle)).unwrap();
    fixture_ready(&mut stream);
    stream.set_read_timeout(Some(POLL)).unwrap();
    assert_eq!(read_expected(&mut stream, b"herdr-remote"), b"herdr-remote");
    let mut byte = [0];
    let error = stream.read(&mut byte).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    // The banner never finishes; cancelling returns without the deadline.
    let stop = AtomicBool::new(true);
    assert!(matches!(
        await_ready(&mut stream, &stop, Instant::now()),
        Err(crate::Error::Ssh(Error::Cancelled))
    ));
    drop(child);
}

#[test]
fn a_banner_without_a_ready_line_is_bounded() {
    let (mut stream, _child) = spawn_bridge(fixture(Case::Oversized)).unwrap();
    fixture_ready(&mut stream);
    stream.set_read_timeout(Some(POLL)).unwrap();
    // Start the banner's clock once the owned child runs, as in the handshake
    // test; the 19,999 bytes left still exceed the limit.
    read_expected(&mut stream, b"x");
    assert!(matches!(
        await_ready(&mut stream, &AtomicBool::new(false), Instant::now()),
        Err(crate::Error::Ssh(Error::OutputLimit))
    ));
}
