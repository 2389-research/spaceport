use super::*;
use crate::control::protocol::app::CoreCatalogState;
use sheprd_core_client::{Error as CoreError, Method};

fn catalog(cx: &mut App) -> CoreCatalog {
    let Response::Status(status) = status(&Caller::default(), cx) else {
        panic!("expected status");
    };
    status.core_catalog
}

#[gpui::test]
fn status_distinguishes_missing_stale_empty_and_refused_core_catalog(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(|cx| {
        let missing = catalog(cx);
        assert_eq!(missing.state, CoreCatalogState::NotStarted);
        assert_eq!(missing.machine_count, None);

        core_terminals::set_machine_catalog_pending_for_test(cx);
        let pending = catalog(cx);
        assert_eq!(pending.state, CoreCatalogState::Synchronizing);
        assert_eq!(pending.machine_count, None);

        core_terminals::set_machine_catalog_for_test(cx, Some(vec![]));
        let empty = catalog(cx);
        assert_eq!(empty.state, CoreCatalogState::Current);
        assert_eq!(empty.machine_count, Some(0));
        assert!(empty.epoch.is_some());

        core_terminals::set_machine_catalog_for_test(cx, None);
        let stale = catalog(cx);
        assert_eq!(stale.state, CoreCatalogState::Disconnected);
        assert_eq!(stale.machine_count, None);
        assert!(stale.epoch > empty.epoch);

        core_terminals::set_machine_catalog_error_for_test(
            cx,
            CoreError::Refused {
                method: Method::MachineList,
                kind: Some("fixture".into()),
                message: "private-token".repeat(1_000),
                hint: None,
            },
        );
        let refused = catalog(cx);
        assert_eq!(refused.state, CoreCatalogState::Unavailable);
        assert_eq!(refused.machine_count, None);
        assert_eq!(refused.error.as_deref(), Some("refused"));
        assert!(
            !serde_json::to_string(&refused)
                .unwrap()
                .contains("private-token")
        );
    });
}
