// ABOUTME: Checks enlarged Settings navigation against actual text and layout widths.
// ABOUTME: Preserves a chosen navigation width while its default follows font changes.
use super::*;

#[gpui::test]
fn enlarged_navigation_fits_labels_without_replacing_a_chosen_width(cx: &mut TestAppContext) {
    let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = fixture_new(weak, cx);
        view.config.ui.size = 24.;
        view
    });
    let draw = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    };
    for width in [1100., 680.] {
        cx.simulate_resize(size(px(width), px(900.)));
        draw(cx);
        let labels = cx.update(|window, cx| {
            let config = &view.read(cx).config;
            Section::ALL
                .iter()
                .map(|section| {
                    let label = section.label();
                    let mut run = window.text_style().to_run(label.len());
                    run.font = crate::config::font_for(&config.ui);
                    (
                        *section,
                        window
                            .text_system()
                            .shape_line(label.into(), px(config.ui.size), &[run], None)
                            .width,
                    )
                })
                .collect::<Vec<_>>()
        });
        let label = cx
            .debug_bounds("settings-section-configuration-label")
            .unwrap();
        for (section, needed) in labels {
            assert!(
                needed <= label.size.width,
                "{section:?}: needs {needed:?}, available {label:?}"
            );
        }
    }
    view.update(cx, |view, cx| {
        view.navigation_width
            .drag(crate::panel_resize::Side::Left, Bounds::default(), px(300.));
        view.config.ui.size = 12.;
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view
            .navigation_width
            .width(view.viewport_width)),
        300.
    );
    // The grip sits inside the navigation border.
    let edge = cx.debug_bounds("settings-navigation-resize").unwrap();
    assert!((edge.right() - px(300.)).abs() <= px(1.), "{edge:?}");
    view.update(cx, |view, cx| {
        view.navigation_width.reset();
        cx.notify();
    });
    draw(cx);
    assert_eq!(
        view.read_with(cx, |view, _| view
            .navigation_width
            .width(view.viewport_width)),
        184.
    );
    let edge = cx.debug_bounds("settings-navigation-resize").unwrap();
    assert!((edge.right() - px(184.)).abs() <= px(1.), "{edge:?}");
}
