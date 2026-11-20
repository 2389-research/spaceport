// ABOUTME: Tests generated output markers in the actual decorated TUI cell-row shape.
// ABOUTME: Keeps shell input echo and markers split across rows from satisfying readiness.
use super::{
    peer,
    report::{MARKER, marker_visible},
};
use anyhow::Result;
use sheprd_client::protocol::{CellData, FrameData};

fn frame(text: &str) -> Result<FrameData> {
    let mut symbols: Vec<_> = text.chars().collect();
    symbols.resize(symbols.len().div_ceil(80) * 80, ' ');
    let frame = FrameData {
        width: 80,
        height: (symbols.len() / 80).try_into()?,
        cells: symbols
            .into_iter()
            .map(|symbol| CellData {
                symbol: symbol.to_string(),
                fg: 0,
                bg: 0,
                modifier: 0,
                skip: false,
                hyperlink: None,
            })
            .collect(),
        cursor: None,
        hyperlinks: Vec::new(),
        graphics: Vec::new(),
    };
    frame.validate()?;
    Ok(frame)
}

#[test]
fn observed_generated_marker_with_scrollbar_is_visible() -> Result<()> {
    // Exact final marker row retained by the real private seed diagnostic.
    let observed =
        "SP_PROFILE_READY                                                               ▐";
    assert_eq!(observed.chars().count(), 80);
    assert!(marker_visible(&frame(observed)?));
    Ok(())
}

#[test]
fn undecorated_generated_marker_is_visible() -> Result<()> {
    assert!(marker_visible(&frame(MARKER)?));
    Ok(())
}

#[test]
fn actual_generated_command_echo_is_not_ready_output() -> Result<()> {
    let command = peer::generated_command();
    assert!(!command.contains(MARKER));
    assert!(!marker_visible(&frame(&command)?));
    Ok(())
}

#[test]
fn marker_must_be_contiguous_inside_one_actual_row() -> Result<()> {
    let split = format!("{}SP_PROFILE_READY", " ".repeat(72));
    assert!(!marker_visible(&frame(&split)?));
    Ok(())
}
