use super::*;
use gpui::{StyleRefinement, px};

/// The style a refinement builds, as the element would carry it.
fn style(build: impl FnOnce(StyleRefinement) -> StyleRefinement) -> StyleRefinement {
    build(StyleRefinement::default())
}

#[test]
fn left_to_right_lays_out_as_written() {
    assert_eq!(
        style(|s| s.ps(false, px(4.)).pe(false, px(9.))),
        style(|s| s.pl(px(4.)).pr(px(9.)))
    );
    assert_eq!(style(|s| s.row(false)), style(|s| s.flex().flex_row()));
    assert_eq!(text_start(false), TextAlign::Left);
    assert_eq!(
        icon("icons/chevron-right.svg", false),
        "icons/chevron-right.svg"
    );
    assert_eq!(glyph("\u{25b8}", false), "\u{25b8}");
}

#[test]
fn right_to_left_mirrors_every_side() {
    let pairs: [(StyleRefinement, StyleRefinement); 9] = [
        (style(|s| s.ps(true, px(4.))), style(|s| s.pr(px(4.)))),
        (style(|s| s.pe(true, px(4.))), style(|s| s.pl(px(4.)))),
        (style(|s| s.ms(true, px(4.))), style(|s| s.mr(px(4.)))),
        (style(|s| s.me(true, px(4.))), style(|s| s.ml(px(4.)))),
        (style(|s| s.inset_s(true, px(4.))), style(|s| s.right(px(4.)))),
        (style(|s| s.inset_e(true, px(4.))), style(|s| s.left(px(4.)))),
        (
            style(|s| s.border_s(true, px(1.))),
            style(|s| s.border_r(px(1.))),
        ),
        (
            style(|s| s.border_e(true, px(1.))),
            style(|s| s.border_l(px(1.))),
        ),
        (
            style(|s| s.rounded_e(true, px(3.))),
            style(|s| s.rounded_l(px(3.))),
        ),
    ];
    for (logical, physical) in pairs {
        assert_eq!(logical, physical);
    }
    assert_eq!(style(|s| s.row(true)), style(|s| s.flex().flex_row_reverse()));
    assert_eq!(style(|s| s.text_start(true)), style(|s| s.text_right()));
    assert_eq!(style(|s| s.text_end(true)), style(|s| s.text_left()));
    assert_eq!(text_start(true), TextAlign::Right);
}

#[test]
fn directional_icons_swap_with_their_embedded_mirror() {
    for (path, mirror) in DIRECTIONAL_ICONS {
        assert_eq!(icon(path, true), *mirror);
        assert_eq!(icon(mirror, true), *path);
        for path in [path, mirror] {
            let asset = gpui::AssetSource::load(&spaceport_icons::Icons, path);
            assert!(matches!(asset, Ok(Some(_))), "{path} has no embedded asset");
        }
    }
}

#[test]
fn directional_glyphs_swap_and_repeat_back() {
    for (shown, mirror) in DIRECTIONAL_GLYPHS {
        assert_eq!(glyph(shown, true), *mirror);
        assert_eq!(glyph(mirror, true), *shown);
        assert_ne!(shown, mirror);
    }
    // A glyph that points nowhere is drawn as it is.
    assert_eq!(glyph("A", true), "A");
}
