// ABOUTME: Measures actual composer layout preparation without machine timing budgets.
// ABOUTME: Reuses unchanged glyph geometry while retaining edit, IME, font, and width invalidation.
use super::*;
use gpui::EntityInputHandler;

#[gpui::test]
fn unchanged_repaints_and_caret_moves_do_not_prepare_the_whole_draft_again(
    cx: &mut TestAppContext,
) {
    let (input, cx) = field(cx);
    cx.simulate_input(&"draft with several lines\n".repeat(20));
    draw(cx);
    draw(cx);
    SHAPING_PASSES.set(0);
    for _ in 0..3 {
        input.update(cx, |_, cx| cx.notify());
        draw(cx);
    }
    cx.simulate_keystrokes("left");
    draw(cx);
    assert_eq!(
        SHAPING_PASSES.get(),
        0,
        "unchanged text still performs full layout preparation"
    );
    assert_caret_visible(&input, cx);
}

#[gpui::test]
fn shaping_refreshes_for_edits_width_font_and_composition(cx: &mut TestAppContext) {
    let (input, cx) = field(cx);
    cx.simulate_input("draft");
    draw(cx);
    draw(cx);
    for change in ["edit", "width", "font", "mark", "unmark"] {
        SHAPING_PASSES.set(0);
        match change {
            "edit" => cx.simulate_input(" more"),
            "width" => cx.simulate_resize(size(px(300.), px(500.))),
            _ => cx.update(|window, cx| {
                input.update(cx, |input, cx| match change {
                    "font" => {
                        let mut font = input.font.clone();
                        font.size += 1.;
                        input.set_appearance(font, input.theme.clone(), cx);
                    }
                    "mark" => input.replace_and_mark_text_in_range(None, "é", None, window, cx),
                    _ => input.unmark_text(window, cx),
                })
            }),
        }
        draw(cx);
        assert!(SHAPING_PASSES.get() > 0, "stale layout after {change}");
        assert_caret_visible(&input, cx);
    }
}
