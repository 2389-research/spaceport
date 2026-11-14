// ABOUTME: Pins which connection changes a WSL drop being mapped survives.
// ABOUTME: Covers both checks: sending the paths, and waiting through a frame gap.

use super::*;
use crate::endpoint::fence::changes::{CHANGES, apply};

/// The mapped paths are pasted only on the selected connection the drop
/// started on.
#[gpui::test]
fn mapped_paths_are_pasted_only_on_the_connection_they_were_dropped_on(cx: &mut TestAppContext) {
    let seen = CHANGES.map(|change| {
        let (view, _peer, cx) = mapping(cx);
        view.update(cx, |view, cx| {
            apply(view, 0, change, cx);
            finish(view, &["/mnt/c/a"], cx);
            assert!(view.wsl_paste.is_none(), "{change:?}");
            view.local_error.is_none()
        })
    });
    // Reselect, SwitchAndReturn, ReconnectUnseen, Reconnect, Reboot, Removed.
    assert_eq!(seen, [true, false, false, false, false, false]);
}

/// Mapped paths wait for a frame only while the connection is the same one;
/// otherwise they are refused at once.
#[gpui::test]
fn mapped_paths_wait_through_a_frame_gap_only_on_the_same_connection(cx: &mut TestAppContext) {
    let seen = CHANGES.map(|change| {
        let (view, _peer, cx) = mapping(cx);
        view.update(cx, |view, cx| {
            apply(view, 0, change, cx);
            view.options.surface_size.cols += 1;
            assert!(!view.input_ready(), "{change:?}");
            finish(view, &["/mnt/c/a"], cx);
            view.wsl_paste.is_some()
        })
    });
    assert_eq!(seen, [true, false, false, false, false, false]);
}
