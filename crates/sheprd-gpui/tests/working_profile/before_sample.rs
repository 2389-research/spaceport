// ABOUTME: Checks the final observation consumed before native profile clocks and sampling.
// ABOUTME: Rejects readiness changes between the earlier wait and the captured observation.
use super::*;

fn ready(working: bool) -> Result<(Observation, Targets)> {
    let targets = Targets {
        boot: "private-boot".into(),
        workspace: "private-workspace".into(),
        pane: "generated-pane".into(),
    };
    let groups: Vec<_> = ["generated-tab", "idle-tab"].into_iter().enumerate().map(|(index, tab)| {
        json!({"index":index,"primary":index==0,"shown_terminal":true,"connected":true,
            "bounds":[0,0,500,600],"tab":tab,"projected_revision":7,
            "presented":{"boot":targets.boot,"projection_revision":7,"surface_revision":9,
                "columns":80,"rows":24,"pane_ids":[format!("pane-{index}")],
                "nonblank_cells":100,"paired_image_assets":0,"marker_visible":index==0,"current":true}})
    }).collect();
    let observation = serde_json::from_value(
        json!({"request_id":1,"gui_pid":1,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,"native_key":true,"app_active":true,"app_hidden":false,
        "native_activation":"synthetic before-sample contract fixture","workspace":targets.workspace,
        "synthetic_working_panes":if working { vec![targets.pane.clone()] } else { vec![] },"groups":groups}),
    )?;
    Ok((observation, targets))
}

#[test]
fn valid_idle_and_working_captures_allow_sampling() -> Result<()> {
    for working in [false, true] {
        let (observation, targets) = ready(working)?;
        assert!(populated(&observation, &targets));
        validate_before_sample(&observation, &targets, working)?;
    }
    Ok(())
}

#[test]
fn missing_group_before_clocks_refuses_sampling() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation.groups.pop();
    assert!(!populated(&observation, &targets));
    assert!(validate_before_sample(&observation, &targets, false).is_err());
    Ok(())
}

#[test]
fn stale_presented_frame_before_clocks_refuses_sampling() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation.groups[0]
        .presented
        .as_mut()
        .context("fixture presented frame")?
        .current = false;
    assert!(!populated(&observation, &targets));
    assert!(validate_before_sample(&observation, &targets, false).is_err());
    Ok(())
}

#[test]
fn lost_working_status_before_clocks_refuses_sampling() -> Result<()> {
    let (mut observation, targets) = ready(true)?;
    observation.synthetic_working_panes.clear();
    assert!(populated(&observation, &targets));
    assert!(validate_before_sample(&observation, &targets, true).is_err());
    Ok(())
}

#[test]
fn unexpected_working_status_before_idle_clocks_refuses_sampling() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation
        .synthetic_working_panes
        .push(targets.pane.clone());
    assert!(populated(&observation, &targets));
    assert!(validate_before_sample(&observation, &targets, false).is_err());
    Ok(())
}
