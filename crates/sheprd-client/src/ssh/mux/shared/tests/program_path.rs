// ABOUTME: Keeps mux executable paths lossless across watchdog wrapping.
// ABOUTME: Exercises an executable filename containing non-UTF-8 bytes.
use super::*;
use std::os::unix::ffi::OsStringExt;

#[test]
fn a_non_utf8_executable_path_can_start_its_private_master() {
    let (_registry, fixture) = fake("non-utf8", READY);
    let program = fixture.dir.join(OsString::from_vec(b"ssh-\xff".to_vec()));
    fs::rename(fixture.dir.join("ssh"), &program).unwrap();
    let registry = Registry::new(program, IDLE);
    let channel = open(&registry, Lane::Quick).unwrap();
    let pid = fixture.pid(&channel.socket);
    drop(channel);
    drop(registry);
    wait_until(|| !alive(&pid));
}
