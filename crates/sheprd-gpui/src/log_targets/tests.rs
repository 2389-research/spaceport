// ABOUTME: Checks the diagnostic log keeps the app's own tracing namespace.
// ABOUTME: Keeps the compiled GUI crate name and the workspace prefix in the collector filter.
use super::*;

#[test]
fn the_app_passes_its_own_crate_and_the_workspace_prefix() {
    assert_eq!(LOG_TARGETS.roots, [env!("CARGO_CRATE_NAME")]);
    // Events here are emitted under this module's path, which starts there.
    assert_eq!(
        module_path!().split("::").next(),
        Some(env!("CARGO_CRATE_NAME"))
    );
    // A workspace crate's paths start with the prefix.
    assert!(std::any::type_name::<Targets>().starts_with(LOG_TARGETS.crate_prefix));
    assert!(!LOG_TARGETS.crate_prefix.is_empty());
}
