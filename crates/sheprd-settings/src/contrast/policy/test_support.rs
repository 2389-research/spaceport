// ABOUTME: Scopes accessibility policy overrides to the current test thread.
// ABOUTME: Restores nested snapshots without mutating the shared process policy.
use super::OVERRIDE;

pub struct Scope(Option<u8>);

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self(OVERRIDE.replace(Some(0)))
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        OVERRIDE.set(self.0);
    }
}
