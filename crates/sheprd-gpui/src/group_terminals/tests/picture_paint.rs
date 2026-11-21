// ABOUTME: Checks cell painting for a populated terminal picture.
// ABOUTME: Requires settled draws to replay the existing scene.
use super::{fixture_window_on_t0, surface};
use crate::{
    SheprdWindow,
    browser::{Pick, Slot},
    controls::Command,
    group_terminals::Shown,
    performance::Counts,
    sidebar::layout_tests::full_draw,
};
use gpui::{Context, Entity, TestAppContext, VisualTestContext, rgb};
use sheprd_client::protocol::CellData;
use std::sync::Arc;

mod invalidation;

fn mirroring_t0(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, Slot, &mut VisualTestContext) {
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
    let mirror = view.read_with(cx, |view, _| view.group_slots()[1]);
    let shown = view.read_with(cx, |view, cx| view.group_shown(mirror.id, cx));
    assert_eq!(shown, Shown::Elsewhere("t0".into()));
    (view, mirror, cx)
}

fn draw(cx: &mut VisualTestContext) -> Counts {
    cx.update(|window, cx| {
        *cx.default_global::<Counts>() = Counts::default();
        window.draw(cx).clear(cx);
        *cx.default_global::<Counts>()
    })
}

fn paint_change<T: 'static>(
    cx: &mut VisualTestContext,
    owner: &Entity<T>,
    change: impl FnOnce(&mut T, &mut Context<T>),
) -> Counts {
    owner.update(cx, |owner, cx| {
        // GPUI's test effect flush draws the notified window before update returns.
        *cx.default_global::<Counts>() = Counts::default();
        change(owner, cx);
        cx.notify();
    });
    cx.update(|_, cx| *cx.global::<Counts>())
}

#[gpui::test]
fn settled_picture_replays_without_repainting_cells(cx: &mut TestAppContext) {
    let (_, _, cx) = mirroring_t0(cx);
    let settled = draw(cx);
    assert_eq!(settled.paints, 0, "settled picture repainted: {settled:?}");
    assert_eq!((settled.quads, settled.glyphs), (0, 0), "{settled:?}");
}

#[gpui::test]
fn changed_output_reaches_the_mirror_before_it_replays(cx: &mut TestAppContext) {
    let (view, mirror, cx) = mirroring_t0(cx);
    // GPUI's test selector API requires a static string.
    let selector = Box::leak(mirror.selector("mirror-terminal").into_boxed_str());
    let bounds = cx.debug_bounds(selector).unwrap();
    let color = 0x123456;
    let changed = paint_change(cx, &view, |view, _| {
        let surface = Arc::make_mut(view.live.surface.as_mut().unwrap());
        surface.surface_revision += 1;
        surface.panes[0].content_revision += 1;
        surface.frame.cells[0].bg = 0x0200_0000 | color;
        view.endpoints[view.selected_endpoint].live = view.live.clone();
    });
    let assert_mirror_color = |cx: &mut VisualTestContext| {
        cx.update(|window, _| {
            let bounds = bounds.scale(window.scale_factor());
            assert!(window.painted_quads().iter().any(|quad| {
                quad.background == rgb(color).into() && bounds.contains(&quad.bounds.origin)
            }));
        });
    };
    assert!(changed.paints > 0, "{changed:?}");
    assert_mirror_color(cx);
    let settled = draw(cx);
    assert_eq!(settled.paints, 0, "{settled:?}");
    assert_mirror_color(cx);
}

#[gpui::test]
fn missing_live_frame_replaces_the_mirror_with_a_stand_in(cx: &mut TestAppContext) {
    let (view, mirror, cx) = mirroring_t0(cx);
    let cached = |cx: &mut VisualTestContext| {
        view.read_with(cx, |view, _| {
            view.group_pictures
                .iter()
                .any(|entry| entry.group() == mirror.id)
        })
    };
    assert!(cached(cx));
    view.update(cx, |view, cx| {
        view.live.surface = None;
        view.endpoints[view.selected_endpoint].live = view.live.clone();
        view.presentation.clear();
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, cx| view.group_shown(mirror.id, cx)),
        Shown::Elsewhere("t0".into())
    );
    let selector = Box::leak(mirror.selector("stand-in").into_boxed_str());
    assert!(cx.debug_bounds(selector).is_some());
    assert!(!cached(cx));
}

#[gpui::test]
fn closing_a_mirror_retires_its_picture(cx: &mut TestAppContext) {
    let (view, mirror, cx) = mirroring_t0(cx);
    assert!(view.read_with(cx, |view, _| {
        view.group_pictures
            .iter()
            .any(|entry| entry.group() == mirror.id)
    }));
    cx.update(|window, cx| view.update(cx, |view, cx| view.close_group(mirror.id, window, cx)));
    draw(cx);
    view.read_with(cx, |view, _| {
        assert!(!view.group_slots().iter().any(|slot| slot.id == mirror.id));
        assert!(view.group_pictures.is_empty());
    });
}
