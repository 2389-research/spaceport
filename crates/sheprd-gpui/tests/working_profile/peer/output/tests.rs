// ABOUTME: Exercises the bounded TUI producer's marker observations over real cell rows.
// ABOUTME: Covers command echo, cropped panes, retained rows, and invalid progress.
use super::*;
use sheprd_client::protocol::{CellData, PaneSurfacePane};

fn frame(rows: &[&str]) -> anyhow::Result<FrameData> {
    let width = 80;
    let height = rows.len().try_into()?;
    let cells = rows
        .iter()
        .flat_map(|row| row.chars().chain(std::iter::repeat(' ')).take(width))
        .map(|symbol| CellData {
            symbol: symbol.to_string(),
            fg: 0,
            bg: 0,
            modifier: 0,
            skip: false,
            hyperlink: None,
        })
        .collect();
    let frame = FrameData {
        width: width.try_into()?,
        height,
        cells,
        cursor: None,
        hyperlinks: Vec::new(),
        graphics: Vec::new(),
    };
    frame.validate()?;
    Ok(frame)
}

fn observe(rows: &[&str], last_step: u8) -> anyhow::Result<Result<Progress, Error>> {
    let frame = frame(rows)?;
    let pane = SurfaceRect {
        x: 0,
        y: 0,
        width: frame.width,
        height: frame.height,
    };
    Ok(observe_frame(&frame, pane, progress(last_step)))
}

fn progress(last_step: u8) -> Progress {
    Progress {
        last_step,
        complete: false,
    }
}

#[test]
fn generated_seed_and_authored_command_echo_do_not_prove_a_step() -> anyhow::Result<()> {
    let command = command();
    let mut rows: Vec<_> = command
        .as_bytes()
        .chunks(80)
        .map(str::from_utf8)
        .collect::<Result<_, _>>()?;
    rows.insert(0, "SP_PROFILE_READY");
    assert_eq!(observe(&rows, 0)??, progress(0));
    Ok(())
}

#[test]
fn partial_rows_and_markers_beyond_the_pane_do_not_prove_a_step() -> anyhow::Result<()> {
    let frame = frame(&[
        "SP_PROFILE_READY SP_OUTPUT:STEP:01!",
        "SP_PROFILE_READY SP_OUTPUT:STEP:01      SP_PROFILE_READY SP_OUTPUT:STEP:01!",
        "!",
    ])?;
    let pane = SurfaceRect {
        x: 0,
        y: 1,
        width: 39,
        height: 2,
    };
    assert_eq!(observe_frame(&frame, pane, progress(0))?, progress(0));
    Ok(())
}

#[test]
fn overlapping_retained_rows_only_advance_for_new_steps() -> anyhow::Result<()> {
    let first = "SP_PROFILE_READY SP_OUTPUT:STEP:01!                                             ▐";
    let second = "SP_PROFILE_READY SP_OUTPUT:STEP:02!";
    assert_eq!(observe(&[first], 0)??, progress(1));
    assert_eq!(observe(&[first, second], 1)??, progress(2));
    assert_eq!(observe(&[first, second], 2)??, progress(2));
    Ok(())
}

#[test]
fn surface_markers_belong_to_one_pane_in_the_expected_boot() -> anyhow::Result<()> {
    let targets = Targets {
        boot: "owned-boot".into(),
        pane: "owned-pane".into(),
        workspace: "owned-workspace".into(),
    };
    let frame = frame(&["SP_PROFILE_READY SP_OUTPUT:STEP:01!"])?;
    let rectangle = SurfaceRect {
        x: 0,
        y: 0,
        width: frame.width,
        height: frame.height,
    };
    let mut surface = PaneSurfaceFrame {
        boot_id: "other-boot".into(),
        projection_revision: 1,
        surface_revision: 1,
        frame,
        panes: vec![PaneSurfacePane {
            pane_id: "other-pane".into(),
            content_revision: 1,
            rect: rectangle,
            inner_rect: rectangle,
            scrollbar_rect: None,
            scroll: None,
            focused: true,
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 0,
            pixel_height: 0,
        }],
        splits: Vec::new(),
        popup: None,
        graphics: Default::default(),
    };
    assert!(matches!(
        observe_surface(&surface, &targets, progress(0)),
        Err(Error::BootChanged)
    ));
    surface.boot_id.clone_from(&targets.boot);
    assert!(matches!(
        observe_surface(&surface, &targets, progress(0)),
        Err(Error::PaneChanged)
    ));
    surface.panes[0].pane_id.clone_from(&targets.pane);
    assert_eq!(
        observe_surface(&surface, &targets, progress(0))?,
        progress(1)
    );
    surface.panes.push(surface.panes[0].clone());
    assert!(matches!(
        observe_surface(&surface, &targets, progress(0)),
        Err(Error::PaneChanged)
    ));
    Ok(())
}

#[test]
fn skipped_or_regressed_steps_are_refused() -> anyhow::Result<()> {
    assert!(matches!(
        observe(&["SP_PROFILE_READY SP_OUTPUT:STEP:03!"], 1)?,
        Err(Error::Sequence {
            expected: 2,
            observed: 3
        })
    ));
    assert!(matches!(
        observe(&["SP_PROFILE_READY SP_OUTPUT:STEP:02!"], 3)?,
        Err(Error::Regression {
            previous: 3,
            observed: 2
        })
    ));
    Ok(())
}

#[test]
fn complete_markers_require_a_known_kind_and_two_in_range_digits() -> anyhow::Result<()> {
    for invalid in [
        "STEP:00", "STEP:65", "STEP:1", "STEP:+1", "STEP:1x", "OTHER:01",
    ] {
        let row = format!("SP_PROFILE_READY SP_OUTPUT:{invalid}!");
        assert!(matches!(observe(&[&row], 0)?, Err(Error::MalformedMarker)));
    }
    Ok(())
}

#[test]
fn the_distinct_completion_marker_requires_all_steps() -> anyhow::Result<()> {
    let final_step = "SP_PROFILE_READY SP_OUTPUT:STEP:64!";
    let complete = "SP_PROFILE_READY SP_OUTPUT:DONE:64!";
    assert_eq!(observe(&[final_step], 63)??, progress(64));
    assert_eq!(
        observe(&[final_step, complete], 63)??,
        Progress {
            last_step: 64,
            complete: true
        }
    );
    for rows in [
        vec![complete],
        vec!["SP_PROFILE_READY SP_OUTPUT:DONE:63!"],
        vec![final_step, complete, complete],
        vec![final_step, complete, final_step],
    ] {
        assert!(matches!(observe(&rows, 63)?, Err(Error::Completion)));
    }
    Ok(())
}
