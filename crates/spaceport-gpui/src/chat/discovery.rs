//! Which panes run a chat program this window may attach to, and how each of
//! them is reached. The route, and the trust boundary that decides it, are
//! `spaceport_chat_core::discovery`'s; this module names the panes.

use super::{MAX_CHATS, Route, Source, Surface, Target};
pub(crate) use spaceport_chat_core::discovery::route;
pub(crate) use spaceport_chat_core::endpoint;
use spaceport_client::protocol::ClientShellSnapshot;
use std::collections::HashSet;

/// The chat panes of a reachable daemon's snapshot, in its order, one per
/// pane and at most [`MAX_CHATS`], each reached by `route`. A daemon with
/// no route has none.
pub(crate) fn targets(snapshot: &ClientShellSnapshot, route: Option<&Route>) -> Vec<Target> {
    let Some(route) = route else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    snapshot
        .agents
        .iter()
        .filter_map(|agent| {
            Some(Target {
                pane_id: agent.pane_id.clone(),
                route: route.clone(),
                source: Source::Program(endpoint(&agent.tokens)?),
                surface: Surface::Herdr,
            })
        })
        .filter(|target| seen.insert(target.pane_id.clone()))
        .take(MAX_CHATS)
        .collect()
}

/// Verified CLI bindings add rich views to existing terminals. Real ACP
/// endpoints win, and a vanished/changed agent cannot retain its old overlay.
pub(crate) fn with_existing(
    mut targets: Vec<Target>,
    snapshot: &ClientShellSnapshot,
    route: Option<&Route>,
    bindings: &[super::existing::Binding],
) -> Vec<Target> {
    let Some(route) = route else { return targets };
    extend(&mut targets, bindings, |binding| {
        if !snapshot.agents.iter().any(|agent| {
            agent.pane_id == binding.pane_id && agent.agent.as_deref() == Some(binding.agent.name())
        }) {
            return None;
        }
        // Only a session named by a UUID is observed, as the probe that
        // made the binding requires too.
        uuid::Uuid::parse_str(&binding.session_id).ok()?;
        Some(Target {
            pane_id: binding.pane_id.clone(),
            route: route.clone(),
            source: Source::Observed(binding.clone()),
            surface: Surface::Herdr,
        })
    });
    targets
}

/// Trusted, explicit sidecars add a view to an existing agent without writing
/// its lifecycle metadata. ACP's own endpoint always takes precedence.
pub(crate) fn with_mirrors(
    mut targets: Vec<Target>,
    snapshot: &ClientShellSnapshot,
    route: Option<&Route>,
    mirrors: &[crate::config::chat::TranscriptMirror],
    render_existing: bool,
) -> Vec<Target> {
    // A static mirror cannot stand in for a failed/pending live lookup: the
    // pane may already contain a different conversation. Keep its terminal.
    if render_existing {
        return targets;
    }
    let Some(route) = route else { return targets };
    extend(&mut targets, mirrors, |mirror| {
        let matches_host = match (route, mirror.host.as_deref()) {
            (Route::Local, None) => true,
            (Route::Ssh(host), Some(expected)) => host == expected,
            _ => false,
        };
        if !matches_host
            || !snapshot
                .agents
                .iter()
                .any(|agent| agent.pane_id == mirror.pane)
        {
            return None;
        }
        Some(Target {
            pane_id: mirror.pane.clone(),
            route: route.clone(),
            source: Source::Program(mirror.endpoint.parse().ok()?),
            surface: Surface::Herdr,
        })
    });
    targets
}

/// Adds the target `target` makes of each of the first [`MAX_CHATS`]
/// candidates, until there are [`MAX_CHATS`] targets. A pane keeps the
/// target it already has, so earlier sources win.
fn extend<T>(
    targets: &mut Vec<Target>,
    candidates: &[T],
    mut target: impl FnMut(&T) -> Option<Target>,
) {
    for candidate in candidates.iter().take(MAX_CHATS) {
        if targets.len() >= MAX_CHATS {
            break;
        }
        if let Some(target) = target(candidate)
            && !targets.iter().any(|known| known.pane_id == target.pane_id)
        {
            targets.push(target);
        }
    }
}
