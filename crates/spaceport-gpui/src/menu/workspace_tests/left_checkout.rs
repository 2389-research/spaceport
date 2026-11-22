//! Whether a checkout's directory has left the repository Herdr keeps for it.
//! Checkout identity decides that from the endpoint's paths alone, so the
//! sidebar's group folding and the note dialog agree on it.

#[test]
fn checkout_paths_follow_the_endpoint_shape() {
    use crate::worktree::left_checkout;
    let key = "/home/mac/src/herdr-gpui/.git";
    for cwd in [
        "/home/mac/src/herdr-gpui",
        "/home/mac/src/herdr-gpui/",
        "/home/mac/src/herdr-gpui/crates/spaceport-gpui",
    ] {
        assert!(!left_checkout(key, cwd), "{cwd}");
    }
    for cwd in [
        "/home/mac/src/wgsextract-cli",
        "/home/mac/src/herdr-gpui-other",
        "/home/mac",
    ] {
        assert!(left_checkout(key, cwd), "{cwd}");
    }
    // A Windows endpoint may report either separator and any case.
    let windows = r"C:\Users\mac\src\herdr-gpui\.git";
    assert!(!left_checkout(
        windows,
        "c:/users/mac/src/HERDR-GPUI/crates"
    ));
    assert!(left_checkout(windows, r"C:\Users\mac\src\wgsextract-cli"));
    // Nothing is known to have moved without a directory or a `<root>/.git` key.
    assert!(!left_checkout(key, ""));
    assert!(!left_checkout("/srv/git/herdr-gpui.git", "/home/mac"));
    assert!(!left_checkout("/.git", "/home/mac"));
}
