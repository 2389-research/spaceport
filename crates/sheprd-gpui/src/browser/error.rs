// ABOUTME: Defines refusals that protect saved native tab identities.
// ABOUTME: Keeps recovery failures typed until the user-facing display boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Too many tabs are open")]
    Capacity,
    #[error(
        "Saved tabs could not be restored safely. Repair the saved tabs and layout files, then restart Spaceport."
    )]
    RestorationBlocked,
}
