// ABOUTME: Exercises phone SSH keys, trust, and enrollment on isolated resources.
// ABOUTME: A private loopback SSH peer observes authentication and terminal bytes.

use super::*;
use std::io::{Read, Write};
use std::sync::atomic::Ordering;
mod peer;
mod roster;

fn open_bridge(
    host: SshHost,
    auth: SshAuth,
    command: String,
    stop: &AtomicBool,
) -> Result<(UnixStream, transport::BridgeGuard), SshError> {
    transport::open_bridge(host, auth, command, stop, None, None)
}
#[path = "../../tests/support/signer.rs"]
mod signer;
use signer::TestSigner;

#[test]
fn first_use_and_changed_host_key_refuse_before_password() {
    let server = peer::Server::start();
    let stop = AtomicBool::new(false);
    let mut host = server.host();
    host.fingerprint = None;
    let error = open_bridge(host.clone(), server.password(), "cat".into(), &stop).unwrap_err();
    assert!(
        matches!(error, SshError::UnknownHost { fingerprint } if fingerprint == server.fingerprint)
    );
    host.fingerprint = Some("SHA256:old-key".into());
    let error = open_bridge(host, server.password(), "cat".into(), &stop).unwrap_err();
    assert!(
        matches!(error, SshError::ChangedHost { fingerprint } if fingerprint == server.fingerprint)
    );
    assert_eq!(server.attempts.load(Ordering::Acquire), 0);
}

#[test]
fn checking_host_trust_never_authenticates() {
    let server = peer::Server::start();
    let mut host = server.host();
    check_ssh_host(host.clone()).unwrap();
    host.fingerprint = None;
    assert!(
        matches!(check_ssh_host(host.clone()), Err(SshError::UnknownHost { fingerprint }) if fingerprint == server.fingerprint)
    );
    host.fingerprint = Some("SHA256:old-key".into());
    assert!(
        matches!(check_ssh_host(host), Err(SshError::ChangedHost { fingerprint }) if fingerprint == server.fingerprint)
    );
    assert_eq!(server.attempts.load(Ordering::Acquire), 0);
}

#[test]
fn verified_host_authenticates_and_exec_stream_is_binary_clean() {
    let server = peer::Server::start();
    let stop = AtomicBool::new(false);
    let incorrect = SshAuth::Password {
        password: "wrong".into(),
    };
    let result = open_bridge(server.host(), incorrect, "cat".into(), &stop);
    assert!(matches!(result, Err(SshError::Authentication)));
    let (mut stream, _guard) =
        open_bridge(server.host(), server.password(), "cat".into(), &stop).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let bytes = b"\0\xfe\x01herdr\r\n";
    stream.write_all(bytes).unwrap();
    let mut echoed = vec![0; bytes.len()];
    stream.read_exact(&mut echoed).unwrap();
    assert_eq!(&echoed, bytes);
    drop(stream);
    server.wait_for_disconnect();
}

#[test]
fn idle_ssh_bridge_survives_the_servers_inactivity_deadline() {
    let server = peer::Server::with_config(russh::server::Config {
        inactivity_timeout: Some(Duration::from_secs(35)),
        ..Default::default()
    });
    let (mut stream, _guard) = open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(false),
    )
    .unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    // Exercise the production timer with no application bytes on either side.
    std::thread::sleep(Duration::from_secs(37));
    stream.write_all(b"still here").unwrap();
    let mut echo = [0; 10];
    stream.read_exact(&mut echo).unwrap();
    assert_eq!(&echo, b"still here");
    assert_eq!(server.attempts.load(Ordering::Acquire), 1);
}

#[test]
fn cancelled_setup_sends_no_credentials() {
    let server = peer::Server::start();
    let result = open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(true),
    );
    assert!(matches!(result, Err(SshError::Cancelled)));
    assert_eq!(server.attempts.load(Ordering::Acquire), 0);
}

#[test]
fn explicit_enrollment_preserves_existing_keys_and_proves_a_fresh_login() {
    let server = peer::Server::start();
    let key = TestSigner::new();
    let public = enclave_public_key(key.point()).unwrap();
    let keyfile = server.home.join(".ssh/authorized_keys");
    std::fs::create_dir(server.home.join(".ssh")).unwrap();
    std::fs::write(&keyfile, "# existing key without final newline").unwrap();
    let installed = || std::fs::read_to_string(&keyfile).unwrap();
    authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key.clone()).unwrap();
    let first = installed();
    assert!(first.starts_with("# existing key without final newline\n"));
    assert!(first.lines().any(|line| line == public));
    assert_eq!(server.key_logins.load(Ordering::Acquire), 1);
    authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key.clone()).unwrap();
    assert_eq!(installed(), first);
    assert_eq!(server.key_logins.load(Ordering::Acquire), 2);

    let restricted = format!("restrict {public}\n");
    std::fs::write(&keyfile, &restricted).unwrap();
    // The fixture only accepts an exact unrestricted line. The client must
    // preserve the existing restriction even when verification then fails.
    assert!(matches!(
        authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key),
        Err(SshError::UnverifiedEnrollment)
    ));
    assert_eq!(installed(), restricted);
}

#[test]
fn enrollment_rejects_a_redirected_keyfile() {
    let server = peer::Server::start();
    let key = TestSigner::new();
    let existing = server.home.join("other");
    std::fs::write(&existing, "unchanged").unwrap();
    std::fs::create_dir(server.home.join(".ssh")).unwrap();
    std::os::unix::fs::symlink(&existing, server.home.join(".ssh/authorized_keys")).unwrap();
    assert!(matches!(
        authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key),
        Err(SshError::Enrollment)
    ));
    assert_eq!(std::fs::read_to_string(existing).unwrap(), "unchanged");
}

#[test]
fn detach_closes_ssh_while_remote_window_blocks_writes() {
    // The peer grants a single byte of SSH credit and never replenishes it.
    let server = peer::Server::with_window_size(1);
    let (mut stream, guard) = open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(false),
    )
    .unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(b"ab").unwrap();
    let mut echo = [0];
    stream.read_exact(&mut echo).unwrap();
    assert_eq!(echo, [b'a']);
    drop(stream);
    drop(guard);
    server.wait_for_disconnect();
}

mod enclave;
mod inline_enrollment;
mod pty;
