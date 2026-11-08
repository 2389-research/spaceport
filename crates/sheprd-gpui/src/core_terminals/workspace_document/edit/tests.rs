use super::*;
use crate::core_terminals::workspace_document::Document;
use sheprd_core_client::{Error as CoreError, Method};
use uuid::Uuid;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

#[test]
fn unrelated_workspace_edit_can_rebase() {
    let mut base = Document::read(serde_json::Value::Null).unwrap();
    base.create(id(1), "One").unwrap();
    let mut latest = base.clone();
    latest.create(id(2), "Two").unwrap();
    assert!(
        Edit::Rename {
            id: id(1),
            label: "First".into()
        }
        .disjoint(&base, &latest)
    );
}

#[test]
fn same_workspace_edit_cannot_rebase() {
    let mut base = Document::read(serde_json::Value::Null).unwrap();
    base.create(id(1), "One").unwrap();
    let mut latest = base.clone();
    latest.archive(id(1), true).unwrap();
    assert!(
        !Edit::Rename {
            id: id(1),
            label: "First".into()
        }
        .disjoint(&base, &latest)
    );
    assert!(!Edit::Delete { id: id(1) }.disjoint(&base, &latest));
}

#[test]
fn same_workspace_creation_cannot_rebase() {
    let base = Document::read(serde_json::Value::Null).unwrap();
    let mut latest = base.clone();
    latest.create(id(1), "Other writer").unwrap();
    assert!(
        !Edit::Create {
            id: id(1),
            label: "Mine".into()
        }
        .disjoint(&base, &latest)
    );
}

#[test]
fn timeout_is_not_a_conflict_to_retry() {
    assert!(!is_conflict(&CoreError::Timeout(Method::DocSet)));
    assert!(is_conflict(&CoreError::Refused {
        method: Method::DocSet,
        kind: Some("conflict".into()),
        message: "stale version".into(),
    }));
}

#[cfg(unix)]
mod live;
