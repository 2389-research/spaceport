use super::*;

mod drag_phases;
mod exclusive;

/// A second device with three workspaces, never connected.
fn remote(id: &str) -> crate::endpoint::Endpoint {
    let mut endpoint = crate::endpoint::Endpoint::new(
        format!("ssh:{id}"),
        id.to_uppercase(),
        spaceport_client::ConnectTarget::Ssh {
            target: id.into(),
            session: "default".into(),
        },
        true,
    );
    endpoint.live.snapshot = Some(std::sync::Arc::new(crate::sidebar::layout_tests::snapshot(
        3,
    )));
    endpoint
}
