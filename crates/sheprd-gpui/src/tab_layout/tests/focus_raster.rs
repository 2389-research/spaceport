// ABOUTME: Measures the real tab focus painter against arbitrary image-backed colors.
// ABOUTME: Reproduces the captured low-contrast native fill without opening a desktop window.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{browser::GroupIds, config::Backdrop, contrast::ratio};
use gpui::{AppContext, HeadlessAppContext, NoopTextSystem};
use std::sync::Arc;

struct Raster {
    background: u32,
    focus: FocusHandle,
}

impl Render for Raster {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();
        let font = crate::config::Config::default().tabs;
        let context = TabContext {
            fill: Fill::new(
                &theme,
                &Backdrop {
                    opacity: 0.75,
                    ..Default::default()
                },
            ),
            font: &font,
            indicators: Indicators::new(None, Default::default(), false, false, &theme),
            ring: Some(0xcd39ac),
            slot: Slot {
                id: GroupIds::default().next(),
                index: 0,
            },
            theme: &theme,
        };
        context.stop(
            div()
                .id("native-ring")
                .size_full()
                .track_focus(&self.focus)
                .bg(rgb(self.background)),
        )
    }
}

fn capture(context: &mut HeadlessAppContext, handle: WindowHandle<Raster>) -> image::RgbaImage {
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

fn color(image: &image::RgbaImage, x: u32, y: u32) -> u32 {
    let [r, g, b, a] = image.get_pixel(x, y).0;
    assert_eq!(a, 255);
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

#[gpui::test]
#[ignore = "requires explicit macOS Metal offscreen rendering; opens no desktop window"]
fn translucent_tab_focus_pixels_contrast_on_captured_and_extreme_backdrops(
    _cx: &mut TestAppContext,
) {
    let mut context = HeadlessAppContext::with_platform(
        Arc::new(NoopTextSystem::new()),
        Arc::new(()),
        gpui_platform::current_headless_renderer,
    );
    let handle = context
        .open_window(size(px(64.), px(34.)), |_, cx| {
            let focus = cx.focus_handle();
            cx.new(|_| Raster {
                background: 0xd9ccdc,
                focus,
            })
        })
        .unwrap();
    for background in [0xd9ccdc, 0, 0xffffff, 0x777777, 0xcd39ac] {
        context
            .update_window(handle.into(), |root, window, cx| {
                root.downcast::<Raster>()
                    .unwrap()
                    .update(cx, |view, _| view.background = background);
                window.blur(cx);
            })
            .unwrap();
        let before = capture(&mut context, handle);
        context
            .update_window(handle.into(), |root, window, cx| {
                let focus = root.downcast::<Raster>().unwrap().read(cx).focus.clone();
                window.focus(&focus, cx);
                window.dispatch_keystroke(Keystroke::parse("right").unwrap(), cx);
                assert!(focus.is_focused(window));
                assert!(window.last_input_was_keyboard());
            })
            .unwrap();
        let after = capture(&mut context, handle);
        assert_eq!(before.dimensions(), after.dimensions());
        let scale = after.width() / 64;
        assert!(scale > 0);
        let x = after.width() / 2;
        let contrasts =
            [scale, 3 * scale].map(|y| ratio(color(&after, x, y), color(&before, x, y)));
        assert!(
            contrasts.into_iter().any(|contrast| contrast >= 3.),
            "native focus disappears on {background:06x}: ratios={contrasts:?}"
        );
    }
}
