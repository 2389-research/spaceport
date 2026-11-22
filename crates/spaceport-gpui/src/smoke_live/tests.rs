// ABOUTME: Checks the live fixture's external workspace request keeps its cause.
// ABOUTME: Fails before a worker or socket exists for invalid geometry.
use super::create_external_workspace;
use anyhow::{Context as _, Result};
use spaceport_client::{ConnectOptions, ConnectTarget, protocol::ClientSurfaceSize};

#[test]
fn external_workspace_error_retains_client_source() -> Result<()> {
    // Invalid geometry fails before spawning a worker or opening a socket.
    let options = ConnectOptions {
        surface_size: ClientSurfaceSize { cols: 0, rows: 0 },
        ..ConnectOptions::default()
    };
    let error = create_external_workspace(
        ConnectTarget::Socket("/unused-smoke.sock".into()),
        options,
        "fixture-boot".into(),
    )
    .err()
    .context("invalid geometry unexpectedly connected")?;
    assert_eq!(error.to_string(), "connecting external workspace client");
    assert!(matches!(
        error.downcast_ref::<spaceport_client::Error>(),
        Some(spaceport_client::Error::EmptySurface)
    ));
    assert!(format!("{error:#}").contains("screen was asked for with no rows or columns"));
    Ok(())
}
