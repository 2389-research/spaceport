// ABOUTME: Exercises phone enrollment while a terminal uses the same password-authenticated SSH session.
// ABOUTME: Verifies both the fresh key-only proof and continued terminal I/O on enrollment failure.

use super::*;

#[test]
fn enrollment_keeps_the_terminal_and_uses_the_original_password_session() {
    for refuse_key in [false, true] {
        let server = peer::Server::start();
        let key = TestSigner::new();
        if refuse_key {
            std::fs::create_dir(server.home.join(".ssh")).unwrap();
            let public = enclave_public_key(key.point()).unwrap();
            std::fs::write(
                server.home.join(".ssh/authorized_keys"),
                format!("restrict {public}\n"),
            )
            .unwrap();
        }

        let (sender, request) = tokio::sync::oneshot::channel();
        let (mut stream, _guard) = transport::open_bridge(
            server.host(),
            server.password(),
            "cat".into(),
            &AtomicBool::new(false),
            Some(request),
            None,
        )
        .unwrap();
        let result = EnrollmentRequest::submit(sender, key.point(), key);
        if refuse_key {
            assert!(matches!(result, Err(SshError::UnverifiedEnrollment)));
        } else {
            result.unwrap();
        }
        assert_eq!(
            server.key_logins.load(Ordering::Acquire),
            usize::from(!refuse_key)
        );
        assert_eq!(
            server.attempts.load(Ordering::Acquire),
            1,
            "terminal and installer share one password authentication"
        );
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(b"hello").unwrap();
        let mut echo = [0; 5];
        stream.read_exact(&mut echo).unwrap();
        assert_eq!(&echo, b"hello");
    }
}

#[test]
fn disconnect_during_a_key_challenge_releases_the_pending_enrollment() {
    use std::sync::{Mutex, mpsc};

    struct PausedKey {
        key: Arc<TestSigner>,
        started: mpsc::SyncSender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl EnclaveSigner for PausedKey {
        fn sign(&self, message: Vec<u8>) -> Result<Vec<u8>, SshError> {
            self.started.send(()).map_err(|_| SshError::Key)?;
            self.release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .map_err(|_| SshError::Key)?;
            self.key.sign(message)
        }
    }
    let server = peer::Server::start();
    let key = TestSigner::new();
    let (started, challenge) = mpsc::sync_channel(1);
    let (release, waiting) = mpsc::sync_channel(1);
    let paused = Arc::new(PausedKey {
        key: key.clone(),
        started,
        release: Mutex::new(waiting),
    });
    let (sender, request) = tokio::sync::oneshot::channel();
    let (mut stream, guard) = transport::open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(false),
        Some(request),
        None,
    )
    .unwrap();
    let pending =
        std::thread::spawn(move || EnrollmentRequest::submit(sender, key.point(), paused));
    challenge.recv_timeout(Duration::from_secs(5)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(b"live").unwrap();
    let mut bytes = [0; 4];
    stream.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"live");
    drop(guard);
    assert!(matches!(pending.join().unwrap(), Err(SshError::Cancelled)));
    release.send(()).unwrap();
    assert_eq!(server.key_logins.load(Ordering::Acquire), 0);
}
