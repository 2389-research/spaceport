use super::*;
use std::{error::Error as _, path::Path};

#[test]
fn io_context_retains_source_kind_and_cleanup_failure() {
    let error = Error::from(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
        .at_path(Path::new("config.toml"));
    assert_eq!(error.to_string(), "config.toml: denied");
    assert!(
        error
            .source()
            .and_then(|source| source.source())
            .and_then(|source| source.downcast_ref::<io::Error>())
            .is_some_and(|source| source.kind() == io::ErrorKind::PermissionDenied)
    );

    let error = Error::Cleanup {
        source: io::Error::other("write failed"),
        path: "temporary".into(),
        cleanup: io::Error::new(io::ErrorKind::PermissionDenied, "cleanup denied"),
    };
    assert_eq!(
        error.to_string(),
        "write failed; removing temporary: cleanup denied"
    );
    assert!(
        error
            .source()
            .is_some_and(|source| source.is::<io::Error>())
    );
    assert!(
        matches!(error, Error::Cleanup { cleanup, .. } if cleanup.kind() == io::ErrorKind::PermissionDenied)
    );
}

#[test]
fn client_schema_presentation_remains_redacted() -> anyhow::Result<()> {
    let source = serde_json::from_str::<Vec<String>>("{}")
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected schema error"))?;
    let error = Error::from(sheprd_client::Error::CatalogSchema(source));
    assert_eq!(error.to_string(), "invalid endpoint catalog schema");
    assert!(
        error
            .source()
            .and_then(|source| source.source())
            .is_some_and(|source| source.is::<serde_json::Error>())
    );
    Ok(())
}
