// ABOUTME: Qualifies Windows recorder ownership before the fixture starts a CPU trace.
// ABOUTME: Refuses existing or unclassified system sampling without stopping any session.
use anyhow::Result;

pub(super) fn ensure_available(_status: &str, _loggers: &str) -> Result<()> {
    anyhow::bail!("Windows recorder ownership preflight is not implemented")
}

#[cfg(test)]
mod tests;
