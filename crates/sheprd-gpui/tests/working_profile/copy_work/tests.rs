// ABOUTME: Tests before-and-after qualification of passive terminal copy counters.
// ABOUTME: Uses typed observations and the same bounded reply codec as the native harness.
use super::{ConnectionWork, Error, Mode, validate};
use crate::working_profile::{
    report::{self, Frame, Group, Observation, Reply},
    wire,
};
use anyhow::{Context as _, Result, bail};
use serde_json::json;
use sheprd_client::patch_stats::{MutationCounters, ReconstructionCounters, Snapshot};

fn stats(connection_id: u64) -> Snapshot {
    Snapshot {
        connection_id,
        delta: ReconstructionCounters {
            accepted: 4,
            attempts: 6,
            main_cells_copied: 100,
            popup_baselines_unsupported: 1,
            reconstruction_elapsed_ns: 50,
            reconstructions: 5,
        },
        full_surface_messages: 3,
        ordinary: MutationCounters {
            attempts: 9,
            cow_copies: 2,
            cow_elapsed_ns: 35,
            empty_baselines: 1,
            main_cells_copied: 200,
            successes: 8,
        },
        overflow: false,
        reuse: ReconstructionCounters {
            accepted: 5,
            attempts: 8,
            main_cells_copied: 300,
            popup_baselines_unsupported: 0,
            reconstruction_elapsed_ns: 60,
            reconstructions: 7,
        },
        scroll: MutationCounters {
            attempts: 11,
            cow_copies: 3,
            cow_elapsed_ns: 45,
            empty_baselines: 2,
            main_cells_copied: 400,
            successes: 9,
        },
    }
}

fn observation(snapshots: &[Snapshot]) -> Observation {
    let groups = snapshots
        .iter()
        .enumerate()
        .map(|(index, snapshot)| Group {
            core: None,
            index,
            primary: index == 0,
            shown_terminal: true,
            connected: true,
            bounds: [0., 0., 200., 100.],
            tab: Some(format!("tab-{index}")),
            projected_revision: Some(10),
            patch_stats: Some(*snapshot),
            presented: Some(Frame {
                boot: "boot".into(),
                projection_revision: 10,
                surface_revision: 12,
                columns: 40,
                rows: 10,
                pane_ids: vec![format!("pane-{index}")],
                nonblank_cells: 20,
                paired_image_assets: 0,
                marker_visible: false,
                current: true,
                has_popup: Some(false),
            }),
        })
        .collect();
    Observation {
        request_id: 1,
        gui_pid: 121,
        paint_work: None,
        release: true,
        reduce_motion: false,
        mark_layer_render_calls: 0,
        window_id: 2,
        window_active: true,
        window_visible: true,
        native_key: None,
        app_active: None,
        app_hidden: None,
        native_activation: String::new(),
        windows_hwnd: None,
        windows_native: None,
        workspace: Some("workspace".into()),
        synthetic_working_panes: Vec::new(),
        groups,
    }
}

#[test]
fn counter_bearing_reply_preserves_the_full_typed_snapshots() -> Result<()> {
    let snapshots = [stats(41), stats(42)];
    let mut original = observation(&snapshots);
    original.groups[1]
        .presented
        .as_mut()
        .context("fixture frame")?
        .has_popup = Some(true);
    let encoded = report::encode_reply(&Reply::Observation(Box::new(original)))?;
    let Reply::Observation(decoded) = wire::read_reply(encoded.as_slice())? else {
        bail!("profile reply did not contain an observation");
    };
    let decoded_counters: Vec<_> = decoded
        .groups
        .iter()
        .map(|group| group.patch_stats)
        .collect();
    assert_eq!(decoded_counters, snapshots.map(Some));
    for (group, expected) in decoded.groups.iter().zip([false, true]) {
        assert_eq!(
            group.presented.as_ref().context("decoded frame")?.has_popup,
            Some(expected)
        );
    }
    Ok(())
}

#[test]
fn streaming_preserves_raw_records_for_distinct_connections() -> Result<()> {
    let first = stats(41);
    let second = stats(42);
    let mut updated = first;
    updated.ordinary.attempts += 1;
    updated.ordinary.successes += 1;
    updated.ordinary.cow_copies += 1;
    updated.ordinary.main_cells_copied += 400;
    updated.ordinary.cow_elapsed_ns += 100;
    let mut after = observation(&[updated, second]);
    // Group identities come from indexes, even when observation order changes.
    after.groups.swap(0, 1);
    let records = validate(&observation(&[first, second]), &after, Mode::Streaming)?;
    assert_eq!(records.len(), 2);
    assert!(records.contains(&ConnectionWork {
        before: first,
        after: updated
    }));
    assert!(records.contains(&ConnectionWork {
        before: second,
        after: second
    }));
    Ok(())
}

#[test]
fn settled_accepts_unchanged_counters_and_deduplicates_shared_owners() -> Result<()> {
    let snapshot = stats(41);
    let before = observation(&[snapshot, snapshot]);
    let after = observation(&[snapshot, snapshot]);
    assert_eq!(
        validate(&before, &after, Mode::Settled)?,
        vec![ConnectionWork {
            before: snapshot,
            after: snapshot
        }]
    );
    Ok(())
}

#[test]
fn hidden_terminal_groups_contribute_no_counter_records() {
    let snapshot = stats(41);
    let mut before = observation(&[snapshot, stats(42)]);
    let mut after = observation(&[snapshot, stats(42)]);
    for observation in [&mut before, &mut after] {
        observation.groups[1].shown_terminal = false;
        observation.groups[1].patch_stats = None;
    }
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Ok(vec![ConnectionWork {
            before: snapshot,
            after: snapshot
        }])
    );
}

#[test]
fn one_bad_shown_terminal_refuses_the_whole_envelope() -> Result<()> {
    let before = observation(&[stats(41), stats(42)]);
    let mut disconnected = observation(&[stats(41), stats(42)]);
    disconnected.groups[1].connected = false;
    let mut missing = observation(&[stats(41), stats(42)]);
    missing.groups[1].presented = None;
    let mut stale = observation(&[stats(41), stats(42)]);
    stale.groups[1]
        .presented
        .as_mut()
        .context("fixture frame")?
        .current = false;
    let mut blank = observation(&[stats(41), stats(42)]);
    blank.groups[1]
        .presented
        .as_mut()
        .context("fixture frame")?
        .nonblank_cells = 0;
    for invalid in [&disconnected, &missing, &stale, &blank] {
        for (start, end) in [(&before, invalid), (invalid, &before)] {
            assert_eq!(
                validate(start, end, Mode::Settled),
                Err(Error::TerminalPresentation { group: 1 })
            );
        }
    }
    Ok(())
}

#[test]
fn streaming_needs_accepted_work_and_recognizes_each_counter_family() {
    let before = stats(41);
    let updates: &[fn(&mut Snapshot)] = &[
        |stats| {
            stats.ordinary.attempts += 1;
            stats.ordinary.successes += 1;
        },
        |stats| {
            stats.scroll.attempts += 1;
            stats.scroll.successes += 1;
        },
        |stats| {
            stats.delta.attempts += 1;
            stats.delta.accepted += 1;
            stats.delta.reconstructions += 1;
        },
        |stats| {
            stats.reuse.attempts += 1;
            stats.reuse.accepted += 1;
            stats.reuse.reconstructions += 1;
        },
        |stats| stats.full_surface_messages += 1,
    ];
    for update in updates {
        let mut after = before;
        update(&mut after);
        assert_eq!(
            validate(
                &observation(&[before]),
                &observation(&[after]),
                Mode::Streaming
            ),
            Ok(vec![ConnectionWork { before, after }])
        );
    }
}

#[test]
fn streaming_refuses_no_accepted_activity() {
    let snapshot = stats(41);
    let before = observation(&[snapshot]);
    let mut attempted = snapshot;
    attempted.ordinary.attempts += 1;
    attempted.ordinary.cow_copies += 1;
    attempted.ordinary.main_cells_copied += 400;
    attempted.ordinary.cow_elapsed_ns += 100;
    attempted.delta.attempts += 1;
    attempted.delta.reconstructions += 1;
    for ending in [snapshot, attempted] {
        assert_eq!(
            validate(&before, &observation(&[ending]), Mode::Streaming),
            Err(Error::NoAcceptedUpdate)
        );
    }
}

#[test]
fn process_and_window_must_match() {
    let mut before = observation(&[stats(41)]);
    let mut after = observation(&[stats(41)]);
    after.gui_pid += 1;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::ProcessIdentity)
    );
    before.gui_pid = 0;
    after.gui_pid = 0;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::ProcessIdentity)
    );
    before.gui_pid = 121;
    after.gui_pid = before.gui_pid;
    after.window_id += 1;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::WindowIdentity)
    );
    before.window_id = 0;
    after.window_id = 0;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::WindowIdentity)
    );
}

#[test]
fn groups_must_keep_their_tab_and_counter_owner() {
    let before = observation(&[stats(41), stats(42)]);
    let mut after = observation(&[stats(41), stats(42)]);
    after.groups[0].tab = Some("another-tab".into());
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::GroupMapping)
    );
    after.groups[0].tab = Some("tab-0".into());
    after.groups[1].index = 0;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::GroupMapping)
    );
    after.groups[1].index = 1;
    after.groups[1].shown_terminal = false;
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::GroupMapping)
    );
    after.groups[1].shown_terminal = true;
    after.groups[1].patch_stats = Some(stats(43));
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::CounterOwner { group: 1 })
    );
}

#[test]
fn zero_counter_owner_is_unavailable() {
    let observation = observation(&[stats(0)]);
    assert_eq!(
        validate(&observation, &observation, Mode::Settled),
        Err(Error::CounterOwner { group: 0 })
    );
}

#[test]
fn matching_group_indexes_still_need_identified_tabs() {
    let mut observation = observation(&[stats(41)]);
    for tab in [None, Some(String::new())] {
        observation.groups[0].tab = tab;
        assert_eq!(
            validate(&observation, &observation, Mode::Settled),
            Err(Error::GroupMapping)
        );
    }
}

#[test]
fn no_shown_terminal_groups_cannot_qualify() {
    let mut observation = observation(&[stats(41)]);
    observation.groups[0].shown_terminal = false;
    assert_eq!(
        validate(&observation, &observation, Mode::Settled),
        Err(Error::NoTerminalGroups)
    );
}

#[test]
fn unavailable_counter_snapshot_refuses_the_envelope() {
    let before = observation(&[stats(41)]);
    let mut after = observation(&[stats(41)]);
    after.groups[0].patch_stats = None;
    for (start, end) in [(&before, &after), (&after, &before)] {
        assert_eq!(
            validate(start, end, Mode::Settled),
            Err(Error::CounterUnavailable)
        );
    }
}

#[test]
fn overflow_in_either_reading_refuses_the_envelope() {
    let mut overflow = stats(41);
    overflow.overflow = true;
    let valid = observation(&[stats(41)]);
    let invalid = observation(&[overflow]);
    for (before, after) in [(&valid, &invalid), (&invalid, &valid)] {
        assert_eq!(
            validate(before, after, Mode::Settled),
            Err(Error::CounterOverflow { connection_id: 41 })
        );
    }
}

#[test]
fn shared_owners_must_agree_within_each_observation() {
    let baseline = stats(41);
    let mut conflicting = baseline;
    conflicting.full_surface_messages += 1;
    let valid = observation(&[baseline, baseline]);
    let invalid = observation(&[baseline, conflicting]);
    for (before, after) in [(&valid, &invalid), (&invalid, &valid)] {
        assert_eq!(
            validate(before, after, Mode::Settled),
            Err(Error::ConflictingReading { connection_id: 41 })
        );
    }
}

#[test]
fn presented_popup_at_either_endpoint_refuses_the_envelope() -> Result<()> {
    let valid = observation(&[stats(41)]);
    let mut invalid = observation(&[stats(41)]);
    invalid.groups[0]
        .presented
        .as_mut()
        .context("fixture frame")?
        .has_popup = Some(true);
    for (before, after) in [(&invalid, &valid), (&valid, &invalid)] {
        assert_eq!(
            validate(before, after, Mode::Settled),
            Err(Error::PopupState { group: 0 })
        );
    }
    Ok(())
}

#[test]
fn legacy_report_without_popup_state_is_unqualified() -> Result<()> {
    let valid = observation(&[stats(41)]);
    let mut legacy = serde_json::to_value(&valid)?;
    let presented = legacy["groups"][0]["presented"]
        .as_object_mut()
        .context("fixture frame map")?;
    presented.remove("has_popup");
    let legacy = serde_json::from_value(legacy)?;
    for (before, after) in [(&legacy, &valid), (&valid, &legacy)] {
        assert_eq!(
            validate(before, after, Mode::Settled),
            Err(Error::PopupState { group: 0 })
        );
    }
    Ok(())
}

#[test]
fn rejects_new_uses_of_unsupported_popup_baselines() {
    let baseline = stats(41);
    let before = observation(&[baseline]);
    for category in ["delta", "reuse"] {
        let mut updated = baseline;
        let counters = if category == "delta" {
            &mut updated.delta
        } else {
            &mut updated.reuse
        };
        counters.attempts += 1;
        counters.popup_baselines_unsupported += 1;
        assert_eq!(
            validate(&before, &observation(&[updated]), Mode::Settled),
            Err(Error::PopupBaseline { connection_id: 41 }),
            "{category}"
        );
    }
}

#[test]
fn rejects_new_attempts_with_empty_mutation_baselines() {
    let baseline = stats(41);
    let before = observation(&[baseline]);
    for category in ["ordinary", "scroll"] {
        let mut updated = baseline;
        let counters = if category == "ordinary" {
            &mut updated.ordinary
        } else {
            &mut updated.scroll
        };
        counters.attempts += 1;
        counters.empty_baselines += 1;
        assert_eq!(
            validate(&before, &observation(&[updated]), Mode::Settled),
            Err(Error::EmptyBaseline { connection_id: 41 }),
            "{category}"
        );
    }
}

#[test]
fn every_cumulative_counter_must_be_monotonic() -> Result<()> {
    let mut baseline = stats(41);
    // Give both unsupported popup counters a positive historical value to test regression.
    baseline.reuse.popup_baselines_unsupported = 1;
    let before = observation(&[baseline]);
    let encoded = serde_json::to_value(baseline)?;
    let top_level = encoded.as_object().context("fixture counter object")?;
    for (category, counters) in top_level {
        if let Some(fields) = counters.as_object() {
            for (field, value) in fields {
                let mut reversed = encoded.clone();
                reversed[category][field] =
                    json!(value.as_u64().context("fixture counter value")? - 1);
                let after = observation(&[serde_json::from_value(reversed)?]);
                assert_eq!(
                    validate(&before, &after, Mode::Settled),
                    Err(Error::BackwardCounters { connection_id: 41 }),
                    "{category}.{field}"
                );
            }
        }
    }
    let mut reversed = encoded;
    reversed["full_surface_messages"] = json!(baseline.full_surface_messages - 1);
    let after = observation(&[serde_json::from_value(reversed)?]);
    assert_eq!(
        validate(&before, &after, Mode::Settled),
        Err(Error::BackwardCounters { connection_id: 41 }),
        "full_surface_messages"
    );
    Ok(())
}
