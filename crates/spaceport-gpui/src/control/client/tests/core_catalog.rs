use crate::control::{
    client::app::describe_core_catalog,
    protocol::app::{CoreCatalog, CoreCatalogState},
};

#[test]
fn core_catalog_line_marks_missing_count_and_bounds_untrusted_error() {
    let catalog = CoreCatalog {
        epoch: Some(9),
        state: CoreCatalogState::Current,
        machine_count: Some(0),
        error: None,
    };
    assert_eq!(
        describe_core_catalog(&catalog),
        "Core machine catalog, epoch 9: current; 0 machines"
    );
    let stale = CoreCatalog {
        state: CoreCatalogState::Disconnected,
        machine_count: None,
        ..catalog.clone()
    };
    assert!(describe_core_catalog(&stale).contains("catalog stale"));

    let unavailable = CoreCatalog {
        state: CoreCatalogState::Unavailable,
        error: Some(format!("{}\u{1b}", "🙂".repeat(500))),
        ..stale
    };
    let line = describe_core_catalog(&unavailable);
    assert!(!line.contains('\u{1b}'));
    assert!(line.chars().count() < 220);
}
