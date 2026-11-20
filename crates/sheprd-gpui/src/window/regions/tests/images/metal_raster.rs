// ABOUTME: Qualifies terminal image fragments through the actual offscreen Metal renderer.
// ABOUTME: Keeps shader pixel checks separate from AppKit input and foreground performance claims.
use super::*;
use crate::{presentation::Picture, smooth_scroll::Slide, terminal::CellMetrics};
use gpui::{AppContext, HeadlessAppContext, NoopTextSystem, Render, Window, prelude::*, rgb};
use std::time::Duration;

struct Raster {
    view: Entity<SheprdWindow>,
    picture: Picture,
    slide: Option<Slide>,
    look: Look,
}

impl Render for Raster {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (layers, _) = self.view.update(cx, |view, cx| {
            view.terminal_regions(Some(&self.picture), &[], self.slide.clone(), &self.look, cx)
        });
        gpui::div()
            .relative()
            .size_full()
            .bg(rgb(0))
            .children(layers)
    }
}

fn context() -> HeadlessAppContext {
    // Only images and blank cells are drawn; no native glyph claim depends on this text system.
    HeadlessAppContext::with_platform(
        Arc::new(NoopTextSystem::new()),
        Arc::new(()),
        gpui_platform::current_headless_renderer,
    )
}

fn build(
    context: &mut HeadlessAppContext,
    surface: PaneSurfaceFrame,
    images: sheprd_client::SurfaceImages,
    metrics: CellMetrics,
) -> gpui::WindowHandle<Raster> {
    context
        .open_window(gpui::size(gpui::px(80.), gpui::px(48.)), |window, cx| {
            let view = cx.new(|cx| fixture_window(window, cx));
            let theme = crate::config::Theme {
                background: 0,
                ..Default::default()
            };
            view.update(cx, |view, _| {
                view.painter
                    .borrow_mut()
                    .set_appearance(14., metrics.height, theme.clone())
            });
            cx.new(|_| Raster {
                view,
                picture: Picture {
                    frame: Arc::new(surface),
                    images: Arc::new(images),
                },
                slide: None,
                look: Look {
                    metrics,
                    theme,
                    ..look()
                },
            })
        })
        .unwrap()
}

fn draw(context: &mut HeadlessAppContext, handle: gpui::WindowHandle<Raster>) -> image::RgbaImage {
    for _ in 0..3 {
        context
            .update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
        context.run_until_parked();
    }
    context.capture_screenshot(handle.into()).unwrap()
}

fn surface() -> PaneSurfaceFrame {
    let mut surface = main_grid::surface();
    surface.frame = frame(&["    ", "    "]);
    surface.frame.cursor = None;
    surface.panes = vec![pane("a", rect(0, 0, 2, 2)), pane("b", rect(2, 0, 2, 2))];
    surface.panes[0].scroll = Some(sheprd_client::protocol::PaneSurfaceScrollMetrics {
        offset_from_bottom: 10,
        max_offset_from_bottom: 100,
        viewport_rows: 2,
    });
    surface
}

fn image(
    width: u32,
    height: u32,
    data: Vec<u8>,
) -> (SurfaceGraphicsPlacement, sheprd_client::SurfaceImages) {
    let mut placed = placement(SurfaceGraphicsSource::PaneLayer {
        pane_id: "a".into(),
        layer_id: "raster".into(),
    });
    placed.asset.image_width = width;
    placed.asset.image_height = height;
    placed.asset.data_len = data.len() as u64;
    placed.z = 2;
    let assets = [SurfaceGraphicsAsset {
        key: placed.asset.clone(),
        data,
    }]
    .into_iter()
    .collect();
    (placed, assets)
}

#[gpui::test]
#[ignore = "requires explicit macOS Metal offscreen rendering"]
fn metal_pixels_preserve_a_thin_historical_image_fragment(_cx: &mut TestAppContext) {
    let mut context = context();
    let mut previous = surface();
    let (mut placed, assets) = image(1, 2, vec![255, 0, 0, 255, 0, 0, 255, 255]);
    placed.y = 1;
    previous.graphics.placements.push(placed);
    let handle = build(&mut context, previous, assets, CellMetrics::new(16., 16.));
    let original = draw(&mut context, handle);
    let reference = original.get_pixel(16, 38).0;
    assert!(reference[0] > 200 && reference[2] < 40);
    context
        .update_window(handle.into(), |root, _, cx| {
            root.downcast::<Raster>().unwrap().update(cx, |root, _| {
                let previous = root.picture.frame.clone();
                let mut next = (*previous).clone();
                next.panes[0].scroll.as_mut().unwrap().offset_from_bottom = 11;
                next.graphics.placements.clear();
                let mut scroll = crate::smooth_scroll::SmoothScroll::default();
                let now = crate::motion::now();
                assert_eq!(
                    scroll.wheel(&previous, "a", 0.75, now - Duration::from_secs(1)),
                    Some(1)
                );
                scroll.observe(&previous, &next);
                root.slide = scroll.slide(now);
                assert_eq!(root.slide.as_ref().unwrap().offset, -0.25);
                root.picture.frame = Arc::new(next);
            });
        })
        .unwrap();
    let image = draw(&mut context, handle);
    let fragment = image.get_pixel(16, 62).0;
    assert!(
        fragment
            .into_iter()
            .zip(reference)
            .all(|(a, b)| a.abs_diff(b) <= 1),
        "the thin fragment preserves its original sampled pixels: {fragment:?} vs {reference:?}"
    );
    assert_eq!(
        image.get_pixel(16, 70).0,
        [0, 0, 0, 255],
        "pixels inside the translated image but below the stationary pane stay black"
    );
}

#[gpui::test]
#[ignore = "requires explicit macOS Metal offscreen rendering"]
fn metal_pixels_do_not_double_blend_crossed_transparent_fragments(_cx: &mut TestAppContext) {
    let mut context = context();
    let mut surface = surface();
    let (mut placed, assets) = image(8, 8, [64, 0, 0, 128].repeat(64));
    placed.x = 1;
    placed.y = 1;
    placed.cols = 3;
    surface.graphics.placements.push(placed);
    let handle = build(&mut context, surface, assets, CellMetrics::new(16.125, 16.));
    let original = draw(&mut context, handle);
    let reference = original.get_pixel(48, 40).0;
    assert!(reference[0] > 0);
    context
        .update_window(handle.into(), |root, _, cx| {
            root.downcast::<Raster>().unwrap().update(cx, |root, _| {
                root.slide = Some(Slide {
                    pane_id: "a".into(),
                    rect: root.picture.frame.panes[0].inner_rect,
                    offset: -0.75,
                    behind: vec![],
                });
            });
        })
        .unwrap();
    let image = draw(&mut context, handle);
    let inside = image.get_pixel(63, 35).0;
    let outside = image.get_pixel(64, 35).0;
    assert_eq!(inside, reference, "the moving fragment blends once");
    assert_eq!(outside, reference, "the stationary fragment blends once");
    assert_eq!(
        inside, outside,
        "shared snapped masks must blend each device pixel once"
    );
    assert!(
        image.get_pixel(63, 10).0[0] > 0,
        "the inside fragment moved up"
    );
    assert_eq!(image.get_pixel(64, 10).0, [0, 0, 0, 255]);
    assert_eq!(image.get_pixel(63, 55).0, [0, 0, 0, 255]);
    assert!(
        image.get_pixel(64, 55).0[0] > 0,
        "the outside fragment stayed still"
    );
}
