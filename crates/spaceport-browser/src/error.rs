// ABOUTME: Defines failures when reading saved tabs, layouts, and native references.
// ABOUTME: Keeps restoration refusal distinct from exhausted tab identities.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Why saved tabs, layouts, or new tab identities cannot be used safely.
/// Restoration failures preserve the source before allowing replacement writes.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Invalid saved browser tabs")]
    InvalidBrowserTabs,
    #[error("An old source tab must name an absolute file path without control characters")]
    InvalidCodeFile,
    #[error("Invalid saved editor groups")]
    InvalidGroupLayouts,
    #[error(
        "Local pages must be a plain file in a folder that is not hidden and does not hold your home directory."
    )]
    InvalidLocalPage,
    #[error("A saved review tab names a checkout that is not a local absolute path.")]
    InvalidReviewCheckout,
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "Saved tabs or layouts need repair. Check browser-tabs.json and editor-groups.json, then restart Spaceport before opening new tabs."
    )]
    RestorationBlocked,
    #[error(transparent)]
    StateFile(#[from] spaceport_state_file::Error),
    #[error(
        "Saved tab IDs are exhausted. Check browser-tabs.json and editor-groups.json, then restart Spaceport before opening new tabs."
    )]
    TabIdsExhausted,
    #[error(transparent)]
    WebUrl(#[from] spaceport_web_url::Error),
    // keep-sorted end
}
