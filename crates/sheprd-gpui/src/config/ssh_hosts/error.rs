//! Why the `[ssh]` table could not be loaded.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("ssh.skip_host_key_verification: {0:?} is not an SSH target")]
    InvalidTarget(String),
    #[error("ssh.skip_host_key_verification must list at most {0} hosts")]
    TooMany(usize),
    // keep-sorted end
}
