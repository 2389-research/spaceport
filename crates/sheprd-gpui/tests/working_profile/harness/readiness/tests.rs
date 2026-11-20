// ABOUTME: Exercises the mixed workload's real pre-clock readiness consumer.
// ABOUTME: Refuses mismatched or hidden Core projections without claiming visible paint.
use super::*;
use crate::working_profile::report::{Core, Group};
use anyhow::Context as _;
use serde_json::{Value, json};

mod motion;

fn workload(targets: &Targets) -> Workload<'_> {
    Workload::Mixed {
        tui: targets,
        core: CoreTarget {
            core_id: "private-core",
            tab: 17,
            terminal: "private-core-terminal",
        },
    }
}

fn ready(working: bool) -> Result<(Observation, Targets)> {
    let targets = Targets {
        boot: "private-boot".into(),
        workspace: "private-workspace".into(),
        pane: "generated-pane".into(),
    };
    let observation = serde_json::from_value(json!({
        "request_id":1,"gui_pid":1,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,"native_key":true,
        "app_active":true,"app_hidden":false,"native_activation":"pure mixed readiness fixture",
        "workspace":targets.workspace,
        "synthetic_working_panes":if working {vec![targets.pane.clone()]} else {vec![]},
        "groups":[
            {"index":0,"primary":true,"shown_terminal":true,"connected":true,
                "bounds":[0,0,500,600],"tab":"generated-tab","projected_revision":7,
                "presented":{"boot":targets.boot,"projection_revision":7,"surface_revision":9,
                    "columns":80,"rows":24,"pane_ids":[targets.pane],"nonblank_cells":120,
                    "paired_image_assets":0,"marker_visible":true,"current":true}},
            {"index":1,"primary":false,"shown_terminal":false,"connected":false,
                "bounds":[500,0,500,600],"tab":null,"projected_revision":null,"presented":null,
                "core":{"tab":17,"terminal":"private-core-terminal","core_id":"private-core",
                    "connected":true,"observe":false,"ended":false,"failed":false,
                    "last_layout":[506,3,488,594],
                    "projected":{"columns":80,"rows":24,"nonblank_cells":120,"projected_marker":true}}}
        ]
    }))?;
    Ok((observation, targets))
}

fn core(observation: &mut Observation) -> Result<&mut Core> {
    observation.groups[1]
        .core
        .as_mut()
        .context("fixture Core group")
}

fn refused(observation: &Observation, targets: &Targets, working: bool) {
    assert!(
        !workload(targets).populated(observation),
        "accepted invalid mixed readiness: {observation:?}"
    );
    assert!(workload(targets).validate(observation, working).is_err());
}

#[test]
fn mixed_projected_readiness_allows_idle_and_controlled_working() -> Result<()> {
    for working in [false, true] {
        let (observation, targets) = ready(working)?;
        assert!(
            workload(&targets).populated(&observation),
            "the mixed workload needs its exact projected Core and generated TUI"
        );
        workload(&targets).validate(&observation, working)?;
        let encoded = serde_json::to_value(&observation)?;
        assert_eq!(
            encoded["groups"][1]["core"]["projected"]["projected_marker"],
            true
        );
        assert!(encoded["groups"][1]["core"].get("presented").is_none());
    }
    Ok(())
}

#[test]
fn mixed_refuses_a_different_core_tab_or_terminal() -> Result<()> {
    for (field, value) in [
        ("core_id", json!("another-core")),
        ("core_id", Value::Null),
        ("tab", json!(18)),
        ("terminal", json!("another-terminal")),
    ] {
        let (mut observation, targets) = ready(false)?;
        let mut encoded = serde_json::to_value(&observation)?;
        encoded["groups"][1]["core"][field] = value;
        observation = serde_json::from_value(encoded)?;
        refused(&observation, &targets, false);
    }
    Ok(())
}

#[test]
fn mixed_refuses_observer_role_and_unavailable_terminal_states() -> Result<()> {
    for (field, value) in [
        ("observe", true),
        ("connected", false),
        ("ended", true),
        ("failed", true),
    ] {
        let (observation, targets) = ready(false)?;
        let mut encoded = serde_json::to_value(observation)?;
        encoded["groups"][1]["core"][field] = json!(value);
        refused(&serde_json::from_value(encoded)?, &targets, false);
    }
    Ok(())
}

#[test]
fn mixed_refuses_absent_empty_or_unmarked_projected_cells() -> Result<()> {
    for (field, value) in [
        ("columns", json!(0)),
        ("rows", json!(0)),
        ("nonblank_cells", json!(0)),
        ("projected_marker", json!(false)),
    ] {
        let (observation, targets) = ready(false)?;
        let mut encoded = serde_json::to_value(observation)?;
        encoded["groups"][1]["core"]["projected"][field] = value;
        refused(&serde_json::from_value(encoded)?, &targets, false);
    }
    let (mut observation, targets) = ready(false)?;
    core(&mut observation)?.projected = None;
    refused(&observation, &targets, false);
    Ok(())
}

#[test]
fn mixed_refuses_missing_or_invalid_layout_and_group_bounds() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    core(&mut observation)?.last_layout = None;
    refused(&observation, &targets, false);
    for index in [2, 3] {
        for dimension in [0., -1., f32::NAN, f32::INFINITY] {
            let (mut observation, targets) = ready(false)?;
            core(&mut observation)?
                .last_layout
                .as_mut()
                .context("fixture layout")?[index] = dimension;
            refused(&observation, &targets, false);
            let (mut observation, targets) = ready(false)?;
            observation.groups[1].bounds[index] = dimension;
            refused(&observation, &targets, false);
        }
    }
    Ok(())
}

#[test]
fn mixed_refuses_hidden_or_unselected_core_groups() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation.groups.remove(1);
    refused(&observation, &targets, false);
    let (mut observation, targets) = ready(false)?;
    observation.groups[1].core = None;
    refused(&observation, &targets, false);
    let (mut observation, targets) = ready(false)?;
    observation.groups[1].shown_terminal = true;
    refused(&observation, &targets, false);
    let (mut observation, targets) = ready(false)?;
    observation
        .groups
        .push(serde_json::from_value::<Group>(serde_json::to_value(
            &observation.groups[1],
        )?)?);
    refused(&observation, &targets, false);
    Ok(())
}

#[test]
fn mixed_refuses_stale_or_unrelated_tui_content() -> Result<()> {
    for (field, value) in [
        ("boot", json!("another-boot")),
        ("current", json!(false)),
        ("pane_ids", json!(["another-pane"])),
        ("marker_visible", json!(false)),
        ("nonblank_cells", json!(0)),
    ] {
        let (observation, targets) = ready(false)?;
        let mut encoded = serde_json::to_value(observation)?;
        encoded["groups"][0]["presented"][field] = value;
        refused(&serde_json::from_value(encoded)?, &targets, false);
    }
    let (mut observation, targets) = ready(false)?;
    observation.groups[0].connected = false;
    refused(&observation, &targets, false);
    let (mut observation, targets) = ready(false)?;
    observation.workspace = Some("another-workspace".into());
    refused(&observation, &targets, false);
    Ok(())
}

#[test]
fn two_tui_workload_cannot_accept_a_projected_core_group() -> Result<()> {
    let (observation, targets) = ready(false)?;
    assert!(!super::super::populated(&observation, &targets));
    assert!(!Workload::TwoTui(&targets).populated(&observation));
    assert!(
        Workload::TwoTui(&targets)
            .validate(&observation, false)
            .is_err()
    );
    Ok(())
}

#[test]
fn mixed_validation_refuses_inactive_window_or_changed_status() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation.window_active = false;
    assert!(workload(&targets).validate(&observation, false).is_err());
    let (mut observation, targets) = ready(true)?;
    observation.synthetic_working_panes.clear();
    assert!(workload(&targets).validate(&observation, true).is_err());
    Ok(())
}
