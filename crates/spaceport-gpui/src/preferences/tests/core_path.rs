// ABOUTME: Verifies Core chrome preferences use a stable path in Spaceport state.
// ABOUTME: Core identity never shares a legacy socket preference file.

use super::*;

#[core::prelude::v1::test]
fn core_chrome_path_is_stable_and_distinct_from_socket_chrome() {
    let root = Path::new("/state/spaceport/gpui");
    let core = core_path(root, "hub-a");
    assert_eq!(core, core_path(root, "hub-a"));
    assert_ne!(core, core_path(root, "hub-b"));
    assert_ne!(core, endpoint_path(root, Path::new("hub-a")));
    assert!(core.starts_with(root));
}
