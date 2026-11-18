// ABOUTME: Reuses shaped symbols when terminal colours change independently of font metrics.
// ABOUTME: Checks actual background paint and cache invalidation through the same painter.
use super::*;

#[gpui::test]
fn theme_changes_keep_glyphs_and_apply_the_new_background(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|_, _| Empty);
    let first = Theme::default();
    let second = Theme {
        background: 0x123456,
        foreground: 0xabcdef,
        ..first.clone()
    };
    let colours = [first.background, second.background];
    cx.draw(Point::default(), size(px(100.), px(100.)), |_, _| {
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let frame = FrameData {
                    width: 1,
                    height: 1,
                    cells: vec![cell("x")],
                    cursor: None,
                    hyperlinks: vec![],
                    graphics: vec![],
                };
                let mut painter = TerminalPainter::default();
                let face = font("Menlo");
                for (index, theme) in [first, second].into_iter().enumerate() {
                    painter.set_appearance(FONT_SIZE, CELL_HEIGHT, theme);
                    if index > 0 {
                        assert_eq!(
                            painter.glyphs.len(),
                            1,
                            "colour changes retain the shaped symbol"
                        );
                        assert!(painter.cell_width.is_some());
                    }
                    let width = painter.cell_width(&face, window, cx);
                    painter.paint_frame(
                        &frame,
                        bounds.origin + point(px(index as f32 * 30.), px(0.)),
                        None,
                        Ground::Opaque,
                        width,
                        &face,
                        &[],
                        &[],
                        None,
                        None,
                        window,
                        cx,
                    );
                    assert!(painter.glyphs.get(0, "x").is_some());
                }
            },
        )
        .size_full()
    });
    cx.update(|window, _| {
        let quads = window.painted_quads();
        for colour in colours {
            assert!(
                quads
                    .iter()
                    .any(|quad| quad.background == rgb(colour).into())
            );
        }
    });
}
