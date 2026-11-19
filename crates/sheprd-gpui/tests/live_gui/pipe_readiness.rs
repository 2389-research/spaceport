// ABOUTME: Checks readiness against an owned Windows pipe without a desktop or daemon.
// ABOUTME: Requires probes to leave the first connection available for actual client bytes.
use super::*;
use interprocess::os::windows::named_pipe::{PipeListenerOptions, pipe_mode};
use std::io::{Read, Write};
use windows_sys::Win32::System::Pipes::{NMPWAIT_NOWAIT, WaitNamedPipeW};

#[allow(unsafe_code)]
fn available(address: &str) -> bool {
    let address: Vec<_> = address.encode_utf16().chain(Some(0)).collect();
    // SAFETY: the NUL-terminated buffer lives through this bounded, non-consuming call.
    unsafe { WaitNamedPipeW(address.as_ptr(), NMPWAIT_NOWAIT) != 0 }
}

#[test]
fn readiness_preserves_the_first_pipe_instance_for_the_real_client() {
    let sandbox = Sandbox::new();
    let address = format!(r"\\.\pipe\{}", sandbox.socket().to_string_lossy());
    let listener = PipeListenerOptions::new()
        .path(Path::new(&address))
        .nonblocking(true)
        .create_duplex::<pipe_mode::Bytes>()
        .unwrap();
    assert!(available(&address), "the private pipe must start available");
    assert!(sandbox.daemon_listening());
    assert!(
        available(&address),
        "readiness consumed the instance before the real client could connect"
    );
    assert!(sandbox.daemon_listening());
    assert!(
        available(&address),
        "repeated readiness consumed the instance"
    );

    let mut client = sheprd_client::Stream::connect(sandbox.socket()).unwrap();
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    client.write_all(b"ready").unwrap();
    client.flush().unwrap();
    let mut server = listener.accept().unwrap();
    let mut bytes = [0; 5];
    server.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"ready");
}

#[test]
fn missing_pipe_is_not_reported_ready() {
    assert!(!Sandbox::new().daemon_listening());
}
