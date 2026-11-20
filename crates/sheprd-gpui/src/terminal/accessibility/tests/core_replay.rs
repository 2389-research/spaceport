// ABOUTME: Replays captured Core bytes through production damage, frame patches, and text projection.
// ABOUTME: Checks chunked output and narrow canvases without creating a window or runtime.
use super::{CELL_HEIGHT, CELL_WIDTH, text};
use crate::{core_terminals::Screen, terminal::Transcript};
use sheprd_backend::{FrameData, Size, SurfaceUpdate};
use sheprd_terminal::{Damage, Emulator};
use std::sync::Arc;

#[path = "../../../../../test-support/core_output_capture.rs"]
mod capture;
use capture::STREAM;

fn replay(chunk: usize) -> Arc<FrameData> {
    let mut emulator = Emulator::new(Size::new(112, 34).unwrap(), 1_000);
    let mut screen = Screen::default();
    for bytes in STREAM.chunks(chunk) {
        emulator.feed(bytes);
        match emulator.take_damage() {
            Damage::Full(frame) => screen.apply(SurfaceUpdate::Frame(frame)),
            Damage::Rows { rows, cursor } if !rows.is_empty() => {
                screen.apply(SurfaceUpdate::Rows { rows, cursor });
            }
            Damage::Rows { .. } => {}
        }
        screen.frame.as_ref().unwrap().validate().unwrap();
    }
    screen.frame.unwrap()
}

#[test]
fn captured_core_pipeline_preserves_output_and_prompt_across_damage_batches() {
    let expected = replay(STREAM.len());
    for chunk in [1, 2, 7, 63, 127] {
        let actual = replay(chunk);
        assert_eq!(actual, expected, "published frame with chunk size {chunk}");
        for visible_columns in [64_u16, 112] {
            let transcript = Transcript::read_frame(
                &actual,
                [],
                CELL_WIDTH,
                CELL_HEIGHT,
                (f32::from(visible_columns) * CELL_WIDTH, 34. * CELL_HEIGHT),
            )
            .unwrap();
            let output = text(&transcript);
            assert!(
                output.starts_with("PS C:\\Users\\runneradmin\\"),
                "{output:?}"
            );
            assert!(
                output.lines().any(|line| line == "SP_A11Y_7"),
                "chunk={chunk} columns={visible_columns}: {output:?}"
            );
            eprintln!(
                "CORE_PIPELINE chunk={chunk} columns={visible_columns} marker=true prompt=true"
            );
        }
    }
}
