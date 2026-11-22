// ABOUTME: Signs SSH fixtures with an ephemeral P-256 identity through the external callback.
// ABOUTME: Shares the same signing contract between isolated peer and opt-in live tests.
use super::{EnclaveSigner, SshAuth, SshError};
use p256::{
    ecdsa::{Signature, SigningKey, signature::Signer},
    elliptic_curve::Generate,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub(super) struct TestSigner {
    key: SigningKey,
    pub(super) count: AtomicUsize,
}

impl TestSigner {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            key: SigningKey::generate_from_rng(&mut rand::rng()),
            count: AtomicUsize::new(0),
        })
    }

    pub(super) fn point(&self) -> Vec<u8> {
        self.key
            .verifying_key()
            .to_sec1_point(false)
            .as_bytes()
            .to_vec()
    }

    pub(super) fn auth(self: &Arc<Self>) -> SshAuth {
        SshAuth::SecureEnclave {
            public_key: self.point(),
            signer: self.clone(),
        }
    }
}

impl EnclaveSigner for TestSigner {
    fn sign(&self, message: Vec<u8>) -> Result<Vec<u8>, SshError> {
        self.count.fetch_add(1, Ordering::Relaxed);
        let signature: Signature = self.key.sign(&message);
        Ok(signature.to_der().as_bytes().to_vec())
    }
}
