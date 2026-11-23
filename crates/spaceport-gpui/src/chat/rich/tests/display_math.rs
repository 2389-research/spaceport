use super::*;

#[test]
fn equation_zoom_keeps_scaling_above_the_previous_thirty_two_pixel_cap() {
    let normal = draw_math("x^2", MathMode::Display, 32., 0xffffff).unwrap();
    let large = draw_math("x^2", MathMode::Display, 50., 0xffffff).unwrap();
    assert!(large.width > normal.width * 1.4);
    assert!(large.height > normal.height * 1.4);
}

#[test]
fn math_lowercase_ink_has_the_optical_height_of_body_text() {
    for size in [20., 40.] {
        for mode in [MathMode::Inline, MathMode::Display] {
            let drawn = draw_math("x", mode, size, 0xffffff).unwrap();
            let pixels = drawn.texture.as_bytes(0).unwrap();
            let row_bytes = (drawn.width * 2.) as usize * 4;
            let ink_height = pixels
                .chunks_exact(row_bytes)
                .filter(|row| row.as_chunks::<4>().0.iter().any(|pixel| pixel[3] >= 64))
                .count() as f32
                / 2.;
            // Typical sans prose has a lowercase height near .54em. Measure
            // the raster's visible ink, not its transparent box or em size.
            assert!(
                (size * 0.50..=size * 0.60).contains(&ink_height),
                "lowercase ink {ink_height}px for {size}px body text"
            );
        }
    }
}
