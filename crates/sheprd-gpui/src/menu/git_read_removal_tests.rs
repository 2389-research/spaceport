//! The window tick must never launch the removed local Git status worker.

#[test]
fn window_tick_has_no_git_status_poll() {
    let window = include_str!("../window.rs");
    assert!(!window.contains("self.update_git()"));
    assert!(!window.contains("git: git::Git"));
}
