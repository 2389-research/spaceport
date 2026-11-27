//! Provider logos, embedded by `spaceport-icons`; this names the one a
//! provider shows.

/// The logo for a provider id, when one ships.
/// The icon `provider` shows: the one it names, else its own logo, else a
/// generic mark.
pub(crate) fn path(provider: super::Provider) -> &'static str {
    provider
        .meta()
        .icon
        .or_else(|| for_id(provider.id()))
        .unwrap_or("icons/agent-generic.svg")
}

fn for_id(id: &str) -> Option<&'static str> {
    super::icon_paths().find(|path| {
        path.strip_prefix("icons/providers/")
            .and_then(|file| file.strip_suffix(".svg"))
            == Some(id)
    })
}
