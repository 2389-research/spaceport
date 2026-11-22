// ABOUTME: Authenticates SSH with a P-256 signing key held outside the Rust library.
// ABOUTME: Translates the platform's DER signature into an SSH user-authentication signature.
use super::SshError;
use p256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
use russh::{
    Signer,
    keys::{
        HashAlg, PublicKey,
        agent::AgentIdentity,
        ssh_key::{self, encoding::Encode, public::EcdsaPublicKey},
    },
};
use std::sync::Arc;

#[uniffi::export(with_foreign)]
pub trait EnclaveSigner: Send + Sync {
    /// Sign the SSH authentication message using ECDSA with SHA-256 and return ASN.1 DER.
    fn sign(&self, message: Vec<u8>) -> Result<Vec<u8>, SshError>;
}

#[uniffi::export]
pub fn enclave_public_key(public_key: Vec<u8>) -> Result<String, SshError> {
    openssh_key(&verifier(&public_key)?)
        .to_openssh()
        .map_err(|_| SshError::Key)
}

fn verifier(point: &[u8]) -> Result<VerifyingKey, SshError> {
    if point.len() != 65 || point.first() != Some(&4) {
        return Err(SshError::Key);
    }
    VerifyingKey::from_sec1_bytes(point).map_err(|_| SshError::Key)
}

fn openssh_key(key: &VerifyingKey) -> PublicKey {
    PublicKey::new(EcdsaPublicKey::from(key).into(), "")
}

pub(super) struct SshSigner {
    key: VerifyingKey,
    signer: Arc<dyn EnclaveSigner>,
    signed: bool,
}

impl SshSigner {
    pub(super) fn new(point: &[u8], signer: Arc<dyn EnclaveSigner>) -> Result<Self, SshError> {
        Ok(Self {
            key: verifier(point)?,
            signer,
            signed: false,
        })
    }

    pub(super) fn authenticated(self, result: russh::client::AuthResult) -> bool {
        // A peer can accept the initial key offer before requesting its signature.
        self.signed && result.success()
    }

    pub(super) fn public_key(&self) -> PublicKey {
        openssh_key(&self.key)
    }
}

impl From<russh::SendError> for SshError {
    fn from(_: russh::SendError) -> Self {
        Self::Connection
    }
}

impl Signer for SshSigner {
    type Error = SshError;

    async fn auth_sign(
        &mut self,
        identity: &AgentIdentity,
        hash: Option<HashAlg>,
        mut message: Vec<u8>,
    ) -> Result<Vec<u8>, SshError> {
        if hash.is_some()
            || !matches!(identity, AgentIdentity::PublicKey { key, .. } if *key == self.public_key())
        {
            return Err(SshError::Key);
        }
        let signer = self.signer.clone();
        let key = self.key;
        // Keep the SSH setup timeout and cancellation live while the platform uses the hardware.
        let signed_message = tokio::task::spawn_blocking(move || {
            let der = signer.sign(message.clone()).map_err(|_| SshError::Key)?;
            let signature = Signature::from_der(&der).map_err(|_| SshError::Key)?;
            key.verify(&message, &signature)
                .map_err(|_| SshError::Key)?;
            let ssh_signature =
                ssh_key::Signature::try_from(signature).map_err(|_| SshError::Key)?;
            // russh's external signer returns the original signed message followed by one SSH string.
            ssh_signature
                .encode_prefixed(&mut message)
                .map_err(|_| SshError::Key)?;
            Ok::<_, SshError>(message)
        })
        .await
        .map_err(|_| SshError::Key)??;
        self.signed = true;
        Ok(signed_message)
    }
}
