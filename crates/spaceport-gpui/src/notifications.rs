// ABOUTME: Schedules the connection's bounded notices into toasts and the OS.
// ABOUTME: Keeps the evidence, delay, mute, and focus policy the model below leaves to the app.
//! When a connection's notice is shown, and where: the evidence and delay a
//! daemon notice must pass, the mutes that take one away, the toast queue's
//! one global order and bounds, and the OS notification center's handoff. The
//! notices themselves — their queue, their stages, their sanitized text and
//! their boot-fenced targets — are `spaceport-live`'s.
// i18n: migrated
use crate::navigation::NavigationTarget;
pub(crate) use spaceport_client::text::{safe_text, unsafe_char};
pub(crate) use spaceport_live::notifications::{
    GRACE, Notice, PENDING_LIMIT, RECHECK, Stage, Toasts, VISIBLE_LIMIT,
};
use std::time::{Duration, Instant};

pub(crate) mod context;
#[cfg(test)]
mod policy_tests;

/// Endpoint ownership retains the existing generation/inbox fences. This single
/// window-wide scheduler imposes ordering and bounds, independent of host order.
///
/// With system delivery, daemon notices that pass the same evidence and delay
/// policy wait for [`take_system`] instead of the in-app queue. Like Herdr, the
/// active tab suppresses them only while this window is `focused`.
///
/// A daemon notice about a workspace or tab the user muted is dropped at any
/// stage short of being posted to the OS, so muting also takes away a toast
/// already showing. One dropped while muted never comes back on unmuting.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 160 lines")]
pub(crate) fn tick(
    endpoints: &mut [crate::endpoint::Endpoint],
    selected: usize,
    config: crate::config::NotificationConfig,
    mutes: Option<&crate::notification_mutes::Mutes>,
    hidden: bool,
    focused: bool,
    pending_navigation: Option<u64>,
    now: Instant,
) -> bool {
    use crate::config::NotificationDelivery as Delivery;
    use spaceport_client::protocol::{AgentStatus, SemanticNotificationKind as Kind};
    let delivery = config.delivery();
    let system = delivery == Delivery::System;
    let mut changed = false;
    for (index, endpoint) in endpoints.iter_mut().enumerate() {
        // Resolved before the toasts are mutated: `Host::live` borrows the whole
        // endpoint, where the field could be split.
        let snapshot = endpoint.live().latest_snapshot().cloned();
        let snapshot = snapshot.as_deref();
        let silenced = mutes
            .map(|mutes| match snapshot {
                Some(snapshot) => {
                    mutes.silenced(&crate::notification_mutes::Daemon::of(endpoint), snapshot)
                }
                None => mutes.silenced_offline(&endpoint.id),
            })
            .unwrap_or_default();
        endpoint.toasts.entries.retain_mut(|(id, n)| {
            let keep = (|| {
                if !n.client_local
                    && (delivery == Delivery::Off
                        || (n.kind == Kind::Finished && !config.finished)
                        || endpoint
                            .toasts
                            .enabled_since
                            .is_some_and(|cutoff| n.arrived <= cutoff))
                {
                    return false;
                }
                if !n.explicit_position {
                    n.position = config.position;
                }
                if let Some(s) = snapshot {
                    if n.boot.as_ref().is_some_and(|boot| *boot != s.boot_id) {
                        return false;
                    }
                    if !n.initialized && now <= n.arrived + GRACE {
                        n.initialize(s, crate::state::describe);
                    }
                }
                // A posted notice is past muting: it only answers its click.
                if !n.is_posted()
                    && !n.client_local
                    && !silenced.is_empty()
                    && silenced.notice(
                        n.workspace_id.as_deref(),
                        n.tab_id.as_deref(),
                        n.pane_id.as_deref(),
                    )
                {
                    return false;
                }
                let checked = match &mut n.stage {
                    Stage::Posted | Stage::Ready => return true,
                    Stage::Visible { expires, displayed } => {
                        let paused =
                            hidden || (index == selected && pending_navigation == Some(*id));
                        let (last, was_hidden) = std::mem::replace(displayed, (now, paused));
                        if paused || was_hidden {
                            *expires += now.saturating_duration_since(last);
                        }
                        return paused || now < *expires;
                    }
                    Stage::Waiting { checked } => checked,
                };
                if !n.client_local {
                    let delay = if n.kind == Kind::Custom {
                        0
                    } else {
                        config.delay_seconds
                    };
                    if now < n.arrived + Duration::from_secs(delay)
                        || checked.is_some_and(|last| now < (last + RECHECK).min(n.arrived + GRACE))
                    {
                        return true;
                    }
                    *checked = Some(now);
                    if delay > 0 || n.kind == Kind::Finished {
                        if let Some(pane) = n.pane_id.as_deref() {
                            let status = snapshot
                                .and_then(|s| s.agents.iter().find(|a| a.pane_id == pane))
                                .map(|a| a.agent_status);
                            let awaiting = status.is_none()
                                || (n.kind == Kind::Finished
                                    && status == Some(AgentStatus::Working));
                            if awaiting {
                                return now < n.arrived + GRACE;
                            }
                            match n.kind {
                                Kind::NeedsAttention if status != Some(AgentStatus::Blocked) => {
                                    return false;
                                }
                                Kind::Finished if status != Some(AgentStatus::Done) => {
                                    return false;
                                }
                                _ => {}
                            }
                        } else if n.kind == Kind::Finished {
                            return false;
                        }
                    }
                    if index == selected
                        && (!system || focused)
                        && snapshot.is_some_and(|s| match n.suppression_target.as_ref() {
                            Some(NavigationTarget::Tab(tab)) => {
                                s.focused_tab_id.as_ref() == Some(tab)
                            }
                            Some(NavigationTarget::Workspace(w)) => {
                                s.focused_workspace_id.as_ref() == Some(w)
                            }
                            _ => false,
                        })
                    {
                        return false;
                    }
                }
                n.stage = Stage::Ready;
                true
            })();
            changed |= !keep;
            keep
        });
    }
    // Only scalar keys are collected; notification payloads remain endpoint-owned.
    for ready in [false, true] {
        let mut queued: Vec<_> = endpoints
            .iter()
            .enumerate()
            .flat_map(|(index, e)| {
                e.toasts
                    .entries
                    .iter()
                    .filter(move |(_, n)| {
                        (if ready { n.is_ready() } else { n.is_waiting() })
                            && (n.client_local || !system)
                    })
                    .map(move |(id, n)| (n.arrived, n.order, index, *id))
            })
            .collect();
        queued.sort_unstable();
        let visible = endpoints
            .iter()
            .any(|e| e.toasts.entries.iter().any(|(_, n)| n.is_visible()));
        if ready && !visible && !hidden && !queued.is_empty() {
            let (_, _, index, id) = queued.remove(0);
            if let Some((_, n)) = endpoints[index]
                .toasts
                .entries
                .iter_mut()
                .find(|(entry, _)| *entry == id)
            {
                n.promote(now);
                changed = true;
            }
        }
        let limit = PENDING_LIMIT;
        let overflow = queued.len().saturating_sub(limit);
        for &(_, _, index, id) in queued.iter().take(overflow) {
            endpoints[index].toasts.dismiss(id);
            changed = true;
        }
    }
    changed
}

/// A ready daemon notice for the OS notification center, already sanitized
/// and bounded by [`Notice::new`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SystemPost {
    pub endpoint: usize,
    pub id: u64,
    /// Stable per host, boot, and pane, so a newer event replaces the older
    /// one where the platform supports it, as Herdr's TUI does.
    pub tag: String,
    pub title: String,
    pub body: Option<String>,
}

/// Marks every notice [`tick`] made ready for system delivery as posted and
/// returns them in global arrival order. Call only from the poll loop: render
/// also ticks, but must never post.
pub(crate) fn take_system(
    endpoints: &mut [crate::endpoint::Endpoint],
    config: crate::config::NotificationConfig,
) -> Vec<SystemPost> {
    if config.delivery() != crate::config::NotificationDelivery::System {
        return Vec::new();
    }
    let mut posts = Vec::new();
    for (index, endpoint) in endpoints.iter_mut().enumerate() {
        for (id, n) in &mut endpoint.toasts.entries {
            if !n.is_ready() || n.client_local {
                continue;
            }
            n.post();
            let boot = n.boot.as_deref().unwrap_or_default();
            let tag = match &n.pane_id {
                Some(pane) => format!("herdr:{}:{boot}:{pane}", endpoint.id),
                None => format!("herdr:{}:{boot}:#{}", endpoint.id, n.order),
            };
            posts.push((
                (n.arrived, n.order),
                SystemPost {
                    endpoint: index,
                    id: *id,
                    tag,
                    title: n.title.clone(),
                    body: n.body.clone(),
                },
            ));
        }
        // Posted notices only answer clicks; keep the newest few per host.
        let posted = endpoint
            .toasts
            .entries
            .iter()
            .filter(|(_, n)| n.is_posted())
            .count();
        let mut excess = posted.saturating_sub(PENDING_LIMIT);
        endpoint.toasts.entries.retain(|(_, n)| {
            let drop = excess > 0 && n.is_posted();
            excess -= usize::from(drop);
            !drop
        });
    }
    posts.sort_unstable_by_key(|(key, _)| *key);
    posts.into_iter().map(|(_, post)| post).collect()
}

#[cfg(test)]
pub(crate) mod tests;
