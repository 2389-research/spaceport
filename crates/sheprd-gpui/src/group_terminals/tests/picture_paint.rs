// ABOUTME: Checks cell painting for a populated terminal picture.
// ABOUTME: Requires settled draws to replay the existing scene.
use super::{fixture_window_on_t0, surface};
use crate::{
    browser::Pick, controls::Command, group_terminals::Shown, performance::Counts,
    sidebar::layout_tests::full_draw,
};
use sheprd_client::protocol::CellData;
use std::sync::Arc;

#[gpui::test]
fn settled_picture_replays_without_repainting_cells(cx: &mut gpui::TestAppContext) {
    let (view, cx) = fixture_window_on_t0(cx);
    let warmed = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let snapshot = view.live.snapshot.clone().unwrap();
            let mut picture = surface(&snapshot, "p0");
            let frame = &mut Arc::make_mut(&mut picture).frame;
            frame.cells = vec![
                CellData {
                    symbol: "x".into(),
                    fg: 0,
                    bg: 0,
                    modifier: 0,
                    skip: false,
                    hyperlink: None,
                };
                usize::from(frame.width) * usize::from(frame.height)
            ];
            frame.validate().unwrap();
            view.live.surface = Some(picture);
            view.endpoints[view.selected_endpoint].live = view.live.clone();
            view.command(Command::SplitEditor, window, cx);
            let slots = view.group_slots();
            let (left, right) = (slots[0].id, slots[1].id);
            let layout = view.ensure_layout().unwrap();
            layout.show(right, Pick::Herdr("t0".into()));
            layout.activate(left);
            view.set_primary_group(left);
        });
        *cx.default_global::<Counts>() = Counts::default();
        full_draw(window, cx).clear(cx);
        *cx.default_global::<Counts>()
    });
    assert!(warmed.paints > 0 && warmed.glyphs > 0, "{warmed:?}");
    let shown = view.read_with(cx, |view, cx| {
        view.group_shown(view.group_slots()[1].id, cx)
    });
    assert_eq!(shown, Shown::Elsewhere("t0".into()));
    let settled = cx.update(|window, cx| {
        *cx.default_global::<Counts>() = Counts::default();
        window.draw(cx).clear(cx);
        *cx.default_global::<Counts>()
    });
    assert_eq!(settled.paints, 0, "settled picture repainted: {settled:?}");
    assert_eq!((settled.quads, settled.glyphs), (0, 0), "{settled:?}");
}
