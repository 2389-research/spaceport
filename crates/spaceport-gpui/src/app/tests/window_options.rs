// ABOUTME: Verifies main-window constraints and isolated native fixture geometry.
// ABOUTME: Exercises the options producer shared by actual window creation.
use super::super::main_window_options;
use gpui::{Bounds, WindowOptions, point, px, size};

fn options(fixture: bool) -> WindowOptions {
    #[cfg(not(feature = "integration-test"))]
    let _ = fixture;
    main_window_options(
        Bounds::new(point(px(0.), px(0.)), size(px(1200.), px(780.))),
        None,
        #[cfg(feature = "integration-test")]
        fixture,
    )
}

#[test]
fn ordinary_main_windows_keep_their_minimum_size() {
    assert_eq!(
        options(false).window_min_size,
        Some(size(px(640.), px(400.)))
    );
}

#[cfg(feature = "integration-test")]
#[test]
fn isolated_fixture_windows_accept_native_narrow_sizes() {
    let options = options(true);
    for width in [360., 480., 900.] {
        assert!(
            options
                .window_min_size
                .is_none_or(|minimum| minimum.width <= px(width) && minimum.height <= px(460.)),
            "the native fixture must accept {width} by 460 logical pixels"
        );
    }
}
