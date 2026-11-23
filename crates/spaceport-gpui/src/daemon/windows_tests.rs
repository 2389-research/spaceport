// ABOUTME: Exercises Herdr's Windows pipe mapping and connection trust boundary.
// ABOUTME: Uses private fixtures without another account or impersonation privileges.
use super::*;
use interprocess::os::windows::{
    ToWtf16 as _,
    named_pipe::{PipeListener, PipeListenerOptions, pipe_mode},
    security_descriptor::SecurityDescriptor,
};
use std::{
    io::{Read, Write},
    os::windows::io::AsHandle,
    sync::atomic::AtomicU64,
};
use windows_sys::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_INVALID_OWNER, ERROR_PRIVILEGE_NOT_HELD,
};

type Listener = PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>;

fn listener(sddl: &str) -> (PathBuf, Listener) {
    try_listener(sddl).unwrap()
}

fn try_listener(sddl: &str) -> io::Result<(PathBuf, Listener)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let socket = env::temp_dir()
        .join(format!(
            "spaceport-daemon-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
        .join("Windows 用户")
        .join("herdr-client.sock");
    let address = format!(r"\\.\pipe\{}", socket.to_string_lossy());
    let descriptor = SecurityDescriptor::deserialize(&sddl.to_wtf_16().unwrap()).unwrap();
    let listener = PipeListenerOptions::new()
        .path(address)
        .accept_remote(false)
        .nonblocking(true)
        .security_descriptor(Some(descriptor))
        .create_duplex::<pipe_mode::Bytes>()?;
    Ok((socket, listener))
}

fn dial(socket: &Path, auto_start: bool) -> io::Result<Stream> {
    connect_or_start(
        socket,
        &AtomicBool::new(false),
        Duration::ZERO,
        || panic!("an existing or refused pipe must never start a daemon"),
        auto_start,
    )
}

#[test]
fn daemon_pipe_preserves_path_mapping_and_allows_current_owner() {
    let sid = spaceport_client::private_endpoint::user_sid().unwrap();
    for sddl in [
        format!("O:{sid}D:P(A;;GRGW;;;{sid})"),
        format!("O:{sid}D:P(A;;GA;;;SY)(A;;GA;;;OW)"),
    ] {
        let (socket, listener) = listener(&sddl);
        let mut client = dial(&socket, true).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut server = listener.accept().unwrap();
        client.write_all(b"client").unwrap();
        client.flush().unwrap();
        let mut read = [0; 6];
        server.read_exact(&mut read).unwrap();
        assert_eq!(&read, b"client");
        server.write_all(b"daemon").unwrap();
        client.read_exact(&mut read).unwrap();
        assert_eq!(&read, b"daemon");
        assert!(!socket.exists());
    }
}

#[test]
fn daemon_pipe_preserves_access_denied_without_starting_a_daemon() {
    let sid = spaceport_client::private_endpoint::user_sid().unwrap();
    // Deny FILE_WRITE_DATA while preserving read and pipe-instance rights.
    let sddl = format!("O:{sid}D:P(D;;0x00000002;;;{sid})(A;;GRGW;;;{sid})");
    for auto_start in [false, true] {
        let (socket, _listener) = listener(&sddl);
        assert_eq!(
            dial(&socket, auto_start).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}

#[test]
fn daemon_pipe_compares_the_opened_owner_despite_read_write_access() {
    let user = spaceport_client::private_endpoint::user_sid().unwrap();
    for requested_owner in ["", "O:BA"] {
        for auto_start in [false, true] {
            // BA uses an existing group only when this token may assign it as owner.
            // No privilege is enabled and no account is created.
            let (socket, listener) =
                match try_listener(&format!("{requested_owner}D:P(A;;GA;;;{user})")) {
                    Ok(pipe) => pipe,
                    Err(error)
                        if !requested_owner.is_empty()
                            && error.raw_os_error().is_some_and(|code| {
                                matches!(
                                    code as u32,
                                    ERROR_ACCESS_DENIED
                                        | ERROR_INVALID_OWNER
                                        | ERROR_PRIVILEGE_NOT_HELD
                                )
                            }) =>
                    {
                        eprintln!(
                            "PIPE OWNER: group ownership unavailable; auto_start={auto_start}"
                        );
                        continue;
                    }
                    Err(error) => panic!("private pipe creation failed: {error}"),
                };
            let mut unchecked = Stream::connect(&socket).unwrap();
            unchecked
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut peer = listener.accept().unwrap();
            let owner = spaceport_client::private_endpoint::owner_sid(peer.as_handle()).unwrap();
            if !requested_owner.is_empty() {
                assert_ne!(
                    owner, user,
                    "the explicit group owner must differ from the user"
                );
            }
            // The unchecked connector can exchange bytes; ACL denial cannot explain a refusal.
            unchecked.write_all(b"probe").unwrap();
            unchecked.flush().unwrap();
            let mut bytes = [0; 5];
            peer.read_exact(&mut bytes).unwrap();
            assert_eq!(&bytes, b"probe");
            drop(unchecked);
            drop(peer);

            let checked = dial(&socket, auto_start);
            if owner == user {
                let client = checked.unwrap();
                let peer = listener.accept().unwrap();
                drop(client);
                drop(peer);
                eprintln!(
                    "PIPE OWNER: same-owner accepted; mismatch unavailable; auto_start={auto_start}"
                );
            } else {
                assert_eq!(checked.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
                eprintln!("PIPE OWNER: readable mismatched-owner refused; auto_start={auto_start}");
            }
        }
    }
}

#[test]
fn daemon_pipe_limits_the_server_to_identifying_the_client() {
    let sid = spaceport_client::private_endpoint::user_sid().unwrap();
    let (socket, listener) = listener(&format!("O:{sid}D:P(A;;GA;;;{sid})"));
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("identification.txt");
    std::fs::write(&marker, "private fixture").unwrap();
    let mut client = dial(&socket, false).unwrap();
    client
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut server = listener.accept().unwrap();
    client.write_all(b"x").unwrap();
    client.flush().unwrap();
    server.read_exact(&mut [0]).unwrap();
    // The safe guard reverts on every return/unwind path. No token is elevated:
    // this thread only attempts access with the client's restricted identity.
    let impersonation = server.impersonate_client().unwrap();
    let opened = std::fs::read_to_string(&marker);
    drop(impersonation);
    let error = opened.expect_err("the server must not act with the client's identity");
    assert!(matches!(error.raw_os_error(), Some(5 | 1346)), "{error:?}");
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "private fixture");
}
