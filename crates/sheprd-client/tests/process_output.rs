// ABOUTME: Checks reader closure after an isolated executable creates its pipe.
// ABOUTME: Prevents concurrent test forks from temporarily retaining a reader copy.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]
use sheprd_client::process_output::{Retain, capture};
use std::{
    fs::File,
    io::{self, Read, Write},
    process::Child,
    thread,
    time::{Duration, Instant},
};

#[path = "../../test-support/sandbox.rs"]
#[expect(
    dead_code,
    clippy::expect_used,
    reason = "this pipe-only fixture reuses isolation without the sandbox's daemon helpers"
)]
mod sandbox;
use sandbox::{Sandbox, stop_children};

const FIXTURE: &str = "SPACEPORT_CAPTURE_DROP_FIXTURE";
const CLOSED: &str = "READER DROP: closed";

struct OwnedChild(Option<Child>);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        stop_children([&mut self.0]);
    }
}

#[test]
fn dropping_a_capture_closes_a_reader_even_while_its_writer_stays_open() {
    let sandbox = Sandbox::new();
    let mut child = OwnedChild(Some(
        sandbox
            .command(std::env::current_exe().unwrap(), "reader-drop.log")
            .args([
                "--exact",
                "owned_pipe_fixture",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(FIXTURE, "1")
            .spawn()
            .unwrap(),
    ));
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.0.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "owned reader-drop fixture timed out"
        );
        thread::sleep(Duration::from_millis(10));
    };
    let mut log = String::new();
    File::open(sandbox.dir.join("reader-drop.log"))
        .unwrap()
        .take(65_536)
        .read_to_string(&mut log)
        .unwrap();
    assert!(
        status.success(),
        "owned reader-drop fixture failed: {status}\n{log}"
    );
    assert!(
        log.contains(CLOSED),
        "owned reader-drop fixture did not run: {log}"
    );
}

#[test]
#[ignore = "owned fixture subprocess; its parent test provides the private environment"]
fn owned_pipe_fixture() {
    assert_eq!(std::env::var(FIXTURE).as_deref(), Ok("1"));
    // This pipe must be created after exec, outside the suite's parallel forks.
    let (pipe, mut writer) = io::pipe().unwrap();
    let capture = capture(pipe, Retain::Prefix(64)).unwrap();
    drop(capture);
    let result = writer.write(b"closed");
    assert!(
        matches!(&result, Err(error) if error.kind() == io::ErrorKind::BrokenPipe),
        "the joined capture must close its reader: {result:?}"
    );
    println!("{CLOSED}");
}
