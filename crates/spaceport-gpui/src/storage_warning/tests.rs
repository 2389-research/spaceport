use super::*;
use spaceport_client::StorageOperation;
use std::{io, path::PathBuf};

fn storage(path: &str) -> spaceport_client::Error {
    spaceport_client::Error::Storage {
        operation: StorageOperation::Read,
        path: PathBuf::from(path),
        source: Box::new(io::Error::from(io::ErrorKind::PermissionDenied).into()),
    }
}

#[test]
fn finds_the_storage_step_inside_a_wrapping_gui_error() {
    let error = crate::Error::from(storage("/private/wsl.json"));
    assert!(StorageFailure::find(&error).is_some());
}

#[test]
fn ignores_errors_without_a_storage_step() {
    let error = crate::Error::from(spaceport_client::Error::Disconnected);
    assert_eq!(StorageFailure::find(&error), None);
}
