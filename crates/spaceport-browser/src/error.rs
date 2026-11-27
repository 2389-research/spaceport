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
    #[error("Invalid saved editor groups")]
    InvalidGroupLayouts,
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
    // keep-sorted end
}
