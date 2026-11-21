// ABOUTME: Exercises picture invalidation with fixed layout metrics and real scene output.
// ABOUTME: Checks popup clipping and image preparation on unchanged picture draws.
use super::{draw, paint_change, surface};
use crate::{
    SheprdWindow,
    browser::{GroupIds, Slot},
    config::{Theme, font_for},
    group_terminals::tests::fixture_snapshot,
    presentation::Picture,
    sidebar::layout_tests::fixture_window,
    terminal::REVERSED,
    terminal_painter::image_probe::{Phase, Session},
};
use gpui::{
    Context, Entity, IntoElement, Render, TestAppContext, VisualTestContext, Window, div, point,
    prelude::*, px, rgb,
};
use sheprd_client::protocol::{
    CellData, ClientShellPopupSurface, FrameData, SurfaceGraphicsAsset, SurfaceGraphicsAssetKey,
    SurfaceGraphicsFormat, SurfaceGraphicsPlacement, SurfaceGraphicsSource, SurfaceGraphicsTarget,
    SurfaceRect,
};
use std::sync::Arc;

struct Fixture {
    source: Entity<SheprdWindow>,
    slot: Slot,
    picture: Option<Picture>,
}

impl Render for Fixture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let element = self.source.update(cx, |source, cx| {
            let font = font_for(&source.config.terminal);
            // This owner skips prepare_paint, which takes its appearance from config.
            let look = source.terminal_picture_look(font, 20., window.scale_factor());
            let mut previous = std::mem::take(&mut source.group_pictures);
            source.render_terminal_mirror(&mut previous, self.slot, self.picture.clone(), look, cx)
        });
        div()
            .flex()
            .flex_col()
            .w(px(96.))
            .h(px(180.))
            .child(element)
    }
}

fn frame(width: u16, height: u16, background: u32) -> FrameData {
    FrameData {
        width,
        height,
        cells: vec![
            CellData {
                symbol: "x".into(),
                fg: 0,
                bg: background,
                modifier: 0,
                skip: false,
                hyperlink: None,
            };
            usize::from(width) * usize::from(height)
        ],
        cursor: None,
        hyperlinks: vec![],
        graphics: vec![],
    }
}

fn picture() -> Picture {
    let mut surface = surface(&fixture_snapshot(), "p0");
    let surface_data = Arc::make_mut(&mut surface);
    surface_data.frame = frame(20, 8, 0x0212_3456);
    surface_data.panes[0].rect = SurfaceRect {
        x: 0,
        y: 0,
        width: 20,
        height: 8,
    };
    surface_data.panes[0].inner_rect = surface_data.panes[0].rect;
    Picture {
        frame: surface,
        images: Default::default(),
    }
}

fn popup(width: u16, height: u16, color: u32) -> Box<ClientShellPopupSurface> {
    Box::new(ClientShellPopupSurface {
        terminal_id: "popup".into(),
        title: "Picture".into(),
        width: None,
        height: None,
        frame: frame(width, height, 0x0200_0000 | color),
        mouse_reporting: false,
        sgr_pixel_mouse: false,
        pixel_width: 0,
        pixel_height: 0,
    })
}

fn fixture(cx: &mut TestAppContext, picture: Picture) -> (Entity<Fixture>, &mut VisualTestContext) {
    cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| fixture_window(window, cx));
        source.update(cx, |source, _| {
            source.cell_width = 10.;
            source.config.terminal_padding = Default::default();
            source.painter.borrow_mut().set_appearance(
                14.,
                20.,
                Theme {
                    foreground: 0x4182a3,
                    ..Theme::default()
                },
            );
        });
        Fixture {
            source,
            slot: Slot {
                id: GroupIds::default().next(),
                index: 0,
            },
            picture: Some(picture),
        }
    })
}

fn has_color(cx: &mut VisualTestContext, color: u32) -> bool {
    cx.update(|window, _| {
        window
            .painted_quads()
            .iter()
            .any(|quad| quad.background == rgb(color).into())
    })
}

#[gpui::test]
fn actual_painter_appearance_invalidates_a_fixed_picture(cx: &mut TestAppContext) {
    let mut picture = picture();
    for cell in &mut Arc::make_mut(&mut picture.frame).frame.cells {
        cell.modifier = REVERSED;
        cell.bg = 0;
    }
    let (owner, cx) = fixture(cx, picture);
    draw(cx);
    assert!(has_color(cx, 0x4182a3));
    assert_eq!(draw(cx).paints, 0);
    for (font_size, cell_height, foreground) in [
        (14., 20., 0x61b2c3),
        (18., 20., 0x61b2c3),
        (18., 24., 0x61b2c3),
    ] {
        let changed = paint_change(cx, &owner, |owner, cx| {
            owner.source.read(cx).painter.borrow_mut().set_appearance(
                font_size,
                cell_height,
                Theme {
                    foreground,
                    ..Theme::default()
                },
            );
        });
        assert_eq!(changed.paints, 1, "{changed:?}");
        assert!(changed.glyphs > 0, "{changed:?}");
        assert!(!has_color(cx, 0x4182a3));
        cx.update(|window, _| {
            let row_height = px(cell_height).scale(window.scale_factor());
            assert!(window.painted_quads().iter().any(|quad| {
                quad.background == rgb(foreground).into() && quad.bounds.size.height == row_height
            }));
        });
        assert_eq!(draw(cx).paints, 0);
        assert!(has_color(cx, foreground));
    }
}

#[gpui::test]
fn changed_popup_is_clipped_and_disappears_with_its_picture(cx: &mut TestAppContext) {
    let (owner, cx) = fixture(cx, picture());
    draw(cx);
    assert!(has_color(cx, 0x123456));
    let bounds = cx.debug_bounds("mirror-terminal").unwrap();
    let mut old_color = 0x764321;
    for (width, height, color, x, y) in [(8, 2, 0xcb4a92, 60., 60.), (6, 3, 0x427dc9, 70., 50.)] {
        let changed = paint_change(cx, &owner, |owner, _| {
            let picture = owner.picture.as_mut().unwrap();
            Arc::make_mut(&mut picture.frame).popup = Some(popup(width, height, color));
        });
        assert_eq!(changed.paints, 2, "{changed:?}");
        assert!(!has_color(cx, old_color));
        cx.update(|window, _| {
            let scale = window.scale_factor();
            let top_left = (bounds.origin + point(px(x), px(y))).scale(scale);
            let quad = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background == rgb(color).into() && quad.bounds.origin == top_left)
                .unwrap_or_else(|| panic!("the first popup row did not reach its centered origin"));
            assert_eq!(quad.content_mask.bounds, bounds.scale(scale));
            assert!(quad.bounds.right() > quad.content_mask.bounds.right());
        });
        assert_eq!(draw(cx).paints, 0);
        assert!(has_color(cx, color));
        old_color = color;
    }
    let changed = paint_change(cx, &owner, |owner, _| {
        Arc::make_mut(&mut owner.picture.as_mut().unwrap().frame).popup = None;
    });
    assert_eq!(changed.paints, 1, "{changed:?}");
    assert!(!has_color(cx, old_color));
    assert!(has_color(cx, 0x123456));
    let changed = paint_change(cx, &owner, |owner, _| {
        owner.picture = None;
    });
    assert_eq!(changed.paints, 0, "{changed:?}");
    assert!(!has_color(cx, 0x123456));
}

fn image(
    target: SurfaceGraphicsTarget,
    id: u32,
) -> (SurfaceGraphicsPlacement, SurfaceGraphicsAsset) {
    let key = SurfaceGraphicsAssetKey {
        source: SurfaceGraphicsSource::Terminal {
            target,
            image_id: id,
        },
        image_width: 1,
        image_height: 1,
        format: SurfaceGraphicsFormat::Rgba,
        data_len: 4,
        data_fingerprint: u64::from(id),
    };
    (
        SurfaceGraphicsPlacement {
            asset: key.clone(),
            logical_placement_id: id,
            x: 0,
            y: 0,
            cols: 1,
            rows: 1,
            source_x: 0,
            source_y: 0,
            source_width: 0,
            source_height: 0,
            x_offset: 0,
            y_offset: 0,
            z: 0,
            scrollback_offset: 0,
        },
        SurfaceGraphicsAsset {
            key,
            data: vec![50, 100, 150, 255],
        },
    )
}

#[gpui::test]
fn unchanged_graphics_pictures_keep_preparing_main_and_popup_images(cx: &mut TestAppContext) {
    let mut picture = picture();
    let surface = Arc::make_mut(&mut picture.frame);
    surface.popup = Some(popup(8, 2, 0xcb4a92));
    let [(main, main_asset), (popup, popup_asset)] = [
        image(
            SurfaceGraphicsTarget::Pane {
                pane_id: "p0".into(),
            },
            1,
        ),
        image(
            SurfaceGraphicsTarget::Popup {
                terminal_id: "popup".into(),
            },
            2,
        ),
    ];
    surface.graphics.placements = vec![main, popup];
    picture.images = Arc::new([main_asset, popup_asset].into_iter().collect());
    let session = Session::start();
    let (_, cx) = fixture(cx, picture);
    for _ in 0..3 {
        session.reset();
        draw(cx);
        cx.run_until_parked();
    }
    for _ in 0..2 {
        session.reset();
        assert_eq!(draw(cx).paints, 2);
        let epochs = session.preparations();
        assert_eq!(
            epochs.iter().map(|(popup, _)| *popup).collect::<Vec<_>>(),
            [false, true]
        );
        assert_eq!(epochs[1].1, epochs[0].1 + 1);
        assert_eq!(session.popup_leases(), [2]);
        let images = session.painted();
        assert_eq!(images.len(), 2);
        assert!(images.iter().all(|image| image.uploaded));
        cx.update(|window, _| {
            let scene = session.scene(window);
            for target in [false, true] {
                assert!(scene.iter().any(|(phase, _)| {
                    matches!(phase, Phase::Image { popup, .. } if *popup == target)
                }));
            }
        });
    }
}
