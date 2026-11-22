// ABOUTME: Authenticates OpenSSH P-256 identities through a detached signing callback.
// ABOUTME: Exercises enrollment, key mismatch and host pinning against a loopback SSH peer.
use super::*;
#[test]
fn enrolls_and_authenticates_using_only_external_p256_signatures() {
    let server = peer::Server::start();
    let key = TestSigner::new();
    let public = enclave_public_key(key.point()).unwrap();
    assert!(public.starts_with("ecdsa-sha2-nistp256 "));
    assert!(matches!(
        enclave_public_key(vec![4; 65]),
        Err(SshError::Key)
    ));
    authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key.clone()).unwrap();
    assert_eq!(
        std::fs::read_to_string(server.home.join(".ssh/authorized_keys"))
            .unwrap()
            .trim(),
        public
    );
    assert_eq!(server.key_logins.load(Ordering::Acquire), 1);
    let (mut stream, _guard) = open_bridge(
        server.host(),
        key.auth(),
        "cat".into(),
        &AtomicBool::new(false),
    )
    .unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(b"proof").unwrap();
    let mut echo = [0; 5];
    stream.read_exact(&mut echo).unwrap();
    assert_eq!(&echo, b"proof");
    assert_eq!(key.count.load(Ordering::Relaxed), 2);
}

#[test]
fn unexpected_host_never_calls_signer_and_wrong_signer_cannot_authenticate() {
    let server = peer::Server::start();
    let key = TestSigner::new();
    let mut host = server.host();
    host.fingerprint = Some("SHA256:changed".into());
    assert!(matches!(
        open_bridge(host, key.auth(), "cat".into(), &AtomicBool::new(false)),
        Err(SshError::ChangedHost { .. })
    ));
    assert_eq!(key.count.load(Ordering::Relaxed), 0);
    let other = TestSigner::new();
    let mismatch = SshAuth::SecureEnclave {
        public_key: other.point(),
        signer: key.clone(),
    };
    assert!(matches!(
        open_bridge(
            server.host(),
            mismatch,
            "cat".into(),
            &AtomicBool::new(false)
        ),
        Err(SshError::Key)
    ));
    assert_eq!(key.count.load(Ordering::Relaxed), 1);
    assert_eq!(server.key_logins.load(Ordering::Acquire), 0);
}

#[test]
fn refused_hardware_signing_cannot_complete_enrollment() {
    struct UnavailableKey;
    impl EnclaveSigner for UnavailableKey {
        fn sign(&self, _: Vec<u8>) -> Result<Vec<u8>, SshError> {
            Err(SshError::Key)
        }
    }

    let server = peer::Server::start();
    let key = TestSigner::new();
    let result = authorize_enclave_phone(
        server.host(),
        "fixture".into(),
        key.point(),
        Arc::new(UnavailableKey),
    );
    assert!(matches!(result, Err(SshError::UnverifiedEnrollment)));
    assert_eq!(server.key_logins.load(Ordering::Acquire), 0);
    let line = enclave_public_key(key.point()).unwrap();
    let authorized = std::fs::read_to_string(server.home.join(".ssh/authorized_keys")).unwrap();
    assert_eq!(authorized.trim(), line);
    authorize_enclave_phone(server.host(), "fixture".into(), key.point(), key).unwrap();
    assert_eq!(
        std::fs::read_to_string(server.home.join(".ssh/authorized_keys")).unwrap(),
        authorized
    );
    assert_eq!(server.key_logins.load(Ordering::Acquire), 1);
}

#[test]
fn external_auth_requires_a_signature_from_the_current_attempt() {
    use crate::ssh::enclave::SshSigner;
    use russh::{Signer, client::AuthResult};

    let signer = TestSigner::new();
    let attempt = || SshSigner::new(&signer.point(), signer.clone()).unwrap();
    let mut first = attempt();
    let identity = first.public_key().into();
    runtime()
        .unwrap()
        .block_on(first.auth_sign(&identity, None, b"first SSH session".to_vec()))
        .unwrap();
    assert!(first.authenticated(AuthResult::Success));
    assert!(!attempt().authenticated(AuthResult::Success));
}
