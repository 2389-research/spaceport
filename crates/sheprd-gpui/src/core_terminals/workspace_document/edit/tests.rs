#![allow(clippy::unwrap_used, clippy::expect_used)]

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
fn membership_rebases_only_when_target_group_is_unchanged() {
    let mut base = Document::read(serde_json::Value::Null).unwrap();
    base.create(id(1), "One").unwrap();
    let mut unrelated = base.clone();
    unrelated.create(id(2), "Two").unwrap();
    let edit = Edit::AddMember {
        id: id(1),
        core_id: "core-a".into(),
        terminal_id: "t7".into(),
    };
    assert!(edit.disjoint(&base, &unrelated));
    let mut changed = base.clone();
    changed.add_member(id(1), "core-b", "t8").unwrap();
    assert!(!edit.disjoint(&base, &changed));
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
        hint: None,
    }));
}

#[test]
fn rebased_write_timeout_keeps_its_attempted_version() {
    let base = VersionedDocument {
        version: 3,
        value: Document::read(serde_json::Value::Null).unwrap(),
    };
    let edit = Edit::Create {
        id: id(1),
        label: "One".into(),
    };
    let mut writes = Vec::new();
    let result = apply_with(
        base,
        &edit,
        || {
            Ok(VersionedDocument {
                version: 7,
                value: Document::read(serde_json::Value::Null).unwrap(),
            })
        },
        |_, version| {
            writes.push(version);
            if version == 3 {
                Err(CoreError::Refused {
                    method: Method::DocSet,
                    kind: Some("conflict".into()),
                    message: "stale version".into(),
                    hint: None,
                })
            } else {
                Err(CoreError::Timeout(Method::DocSet))
            }
        },
    );
    assert_eq!(writes, vec![3, 7]);
    assert!(matches!(
        result,
        Err(Error::WriteUnknown {
            version: 7,
            source: CoreError::Timeout(Method::DocSet)
        })
    ));
}

#[test]
fn malformed_set_success_is_unknown_but_unsent_errors_are_definite() {
    let malformed = CoreError::Invalid(
        Method::DocSet,
        serde_json::from_value::<u64>(serde_json::Value::Null).unwrap_err(),
    );
    assert!(matches!(
        write_error(7, malformed),
        Error::WriteUnknown {
            version: 7,
            source: CoreError::Invalid(Method::DocSet, _)
        }
    ));
    assert!(matches!(
        write_error(7, CoreError::Busy),
        Error::Core(CoreError::Busy)
    ));
    assert!(matches!(
        write_error(7, CoreError::Unsupported(Method::DocSet)),
        Error::Core(CoreError::Unsupported(Method::DocSet))
    ));
    assert!(matches!(
        write_error(
            7,
            CoreError::Refused {
                method: Method::DocSet,
                kind: Some("internal".into()),
                message: "save failed".into(),
                hint: None,
            }
        ),
        Error::WriteUnknown {
            version: 7,
            source: CoreError::Refused { .. }
        }
    ));
}

#[cfg(unix)]
mod live;
