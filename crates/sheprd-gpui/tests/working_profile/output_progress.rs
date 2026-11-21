// ABOUTME: Checks bounded profile reply encoding for per-pane output progress.
// ABOUTME: Keeps unavailable output metadata distinct from a decoded scalar observation.
use super::{
    report::{self, Frame, Group, Observation, OutputProgress, PaneOutput, Reply},
    wire,
};
use anyhow::{Context as _, Result, bail};
use sheprd_client::protocol::{FrameData, PaneSurfaceFrame};

fn reply(outputs: Option<Vec<PaneOutput>>) -> Reply {
    Reply::Observation(Box::new(Observation {
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
        groups: vec![Group {
            core: None,
            index: 0,
            primary: true,
            shown_terminal: true,
            connected: true,
            bounds: [0., 0., 200., 100.],
            tab: Some("tab".into()),
            projected_revision: Some(10),
            patch_stats: None,
            presented: Some(Frame {
                boot: "boot".into(),
                projection_revision: 10,
                surface_revision: 12,
                columns: 40,
                rows: 10,
                pane_ids: vec!["pane".into()],
                nonblank_cells: 35,
                paired_image_assets: 0,
                marker_visible: true,
                current: true,
                has_popup: Some(false),
                outputs,
            }),
        }],
    }))
}

#[test]
fn output_progress_round_trips_through_the_bounded_reply() -> Result<()> {
    let progress = OutputProgress {
        last_step: 64,
        complete: true,
    };
    let original = reply(Some(vec![PaneOutput {
        pane_id: "pane".into(),
        progress,
    }]));
    let bytes = report::encode_reply(&original)?;
    let decoded = wire::read_reply(bytes.as_slice())?;
    assert_eq!(
        serde_json::to_value(&decoded)?,
        serde_json::to_value(original)?
    );
    let Reply::Observation(decoded) = decoded else {
        bail!("expected a profile observation");
    };
    let frame = decoded.groups[0]
        .presented
        .as_ref()
        .context("presented frame")?;
    assert_eq!(
        frame.outputs,
        Some(vec![PaneOutput {
            pane_id: "pane".into(),
            progress,
        }])
    );
    Ok(())
}

#[test]
fn unavailable_output_progress_is_omitted_and_decodes_as_unavailable() -> Result<()> {
    let invalid = PaneSurfaceFrame {
        boot_id: "boot".into(),
        projection_revision: 10,
        surface_revision: 12,
        frame: FrameData {
            width: 40,
            height: 10,
            cells: Vec::new(),
            cursor: None,
            hyperlinks: Vec::new(),
            graphics: Vec::new(),
        },
        panes: Vec::new(),
        splits: Vec::new(),
        popup: None,
        graphics: Default::default(),
    };
    assert!(invalid.frame.validate().is_err());
    let bytes = report::encode_reply(&reply(report::output::observe(&invalid)))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;
    let frame = &json["data"]["groups"][0]["presented"];
    assert!(frame.is_object() && frame.get("outputs").is_none());
    let Reply::Observation(decoded) = wire::read_reply(bytes.as_slice())? else {
        bail!("expected a profile observation");
    };
    let frame = decoded.groups[0]
        .presented
        .as_ref()
        .context("presented frame")?;
    assert_eq!(frame.outputs, None);
    Ok(())
}
