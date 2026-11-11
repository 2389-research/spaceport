// ABOUTME: Validates the running preview bundle before starting native updates.
// ABOUTME: Keeps build identity and signed bundle trust settings consistent.
pub(super) struct Bundle<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub path: &'a str,
    pub feed: &'a str,
    pub key: &'a str,
    pub signed_feed: bool,
    pub verify_before_extraction: bool,
    pub automatic_install: bool,
    pub automatic_checks_configured: bool,
}

pub(super) fn eligible(
    bundle: &Bundle<'_>,
    id: &str,
    version: &str,
    feed: &str,
    key: &str,
    worktree: &str,
) -> bool {
    worktree == "0"
        && id == "ai.2389.spaceport"
        && version != "dev"
        && !version.is_empty()
        && bundle.id == id
        && bundle.version == version
        && std::path::Path::new(bundle.path)
            .extension()
            .is_some_and(|ext| ext == "app")
        && bundle.feed == feed
        && bundle.key == key
        && bundle.signed_feed
        && bundle.verify_before_extraction
        && !bundle.automatic_install
        && !bundle.automatic_checks_configured
}

#[cfg(test)]
#[path = "tests/policy.rs"]
mod tests;
