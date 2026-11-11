// ABOUTME: Keeps installed updater menu eligibility stable through transient checks.
// ABOUTME: Separately preserves unsupported and live Busy invocation results.
use super::*;

#[test]
fn unsupported_update_menu_stays_disabled() {
    assert!(!menu_enabled(Err(Error::Unsupported)));
}

#[cfg(all(target_os = "macos", feature = "macos-updater"))]
#[test]
fn rebuild_during_busy_check_keeps_menu_enabled_after_returning_idle() {
    let mut disabled = !menu_enabled(Ok(()));
    assert!(!disabled);
    let invocation = Err(Error::Busy);
    assert!(matches!(invocation, Err(Error::Busy)));
    disabled = !menu_enabled(invocation);
    assert!(
        !disabled,
        "rebuilding while busy must not persist a disabled menu"
    );
    // Sparkle becomes idle without another menu rebuild. The stored item still works.
    let invocation: Result<(), Error> = Ok(());
    assert!(invocation.is_ok());
    assert!(!disabled);
    assert!(menu_enabled(invocation));
}

#[cfg(all(target_os = "macos", feature = "macos-updater"))]
#[test]
fn genuine_updater_failure_keeps_menu_disabled() {
    assert!(!menu_enabled(Err(Error::Sparkle(
        sparkle_updater::Error::UpdaterNotReady,
    ))));
}
