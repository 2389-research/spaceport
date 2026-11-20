// ABOUTME: Keeps popup image changes from invalidating cached terminal cells.
// ABOUTME: Exercises cached cell regions while main and popup images stay uncached.
use super::*;
use sheprd_client::protocol::{
    ClientShellPopupSurface, SurfaceGraphicsAsset, SurfaceGraphicsAssetKey, SurfaceGraphicsFormat,
    SurfaceGraphicsPlacement, SurfaceGraphicsSource, SurfaceGraphicsTarget,
};

mod main_grid;
#[cfg(all(target_os = "macos", feature = "integration-test"))]
mod metal_raster;

fn placement(source: SurfaceGraphicsSource) -> SurfaceGraphicsPlacement {
    SurfaceGraphicsPlacement {
        asset: SurfaceGraphicsAssetKey {
            source,
            image_width: 1,
            image_height: 1,
            format: SurfaceGraphicsFormat::Rgba,
            data_len: 4,
            data_fingerprint: 1,
        },
        logical_placement_id: 1,
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
    }
}

#[gpui::test]
fn popup_and_main_images_preserve_unchanged_cell_regions(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let mut surface = PaneSurfaceFrame {
        boot_id: String::new(),
        projection_revision: 0,
        surface_revision: 1,
        frame: frame(&["aaaa|bbbb", "aaaa|bbbb"]),
        panes: vec![pane("a", rect(0, 0, 4, 2)), pane("b", rect(5, 0, 4, 2))],
        splits: vec![],
        popup: Some(Box::new(ClientShellPopupSurface {
            terminal_id: "popup".into(),
            title: "Picture".into(),
            width: None,
            height: None,
            frame: frame(&["x"]),
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            pixel_width: 0,
            pixel_height: 0,
        })),
        graphics: Default::default(),
    };
    let first = draw(&view, &surface, cx);
    assert!(!first.is_empty());
    let popup = placement(SurfaceGraphicsSource::Terminal {
        target: SurfaceGraphicsTarget::Popup {
            terminal_id: "popup".into(),
        },
        image_id: 1,
    });
    view.update(cx, |view, _| {
        view.live.surface_images = Arc::new(
            [SurfaceGraphicsAsset {
                key: popup.asset.clone(),
                data: vec![1, 2, 3, 255],
            }]
            .into_iter()
            .collect(),
        );
    });
    surface.graphics.placements.push(popup);
    assert_eq!(
        draw(&view, &surface, cx),
        first,
        "popup images must not discard unchanged terminal regions"
    );
    cx.run_until_parked();
    assert_eq!(draw(&view, &surface, cx), first);
    for source in [
        SurfaceGraphicsSource::Terminal {
            target: SurfaceGraphicsTarget::Pane {
                pane_id: "a".into(),
            },
            image_id: 2,
        },
        SurfaceGraphicsSource::PaneLayer {
            pane_id: "a".into(),
            layer_id: "picture".into(),
        },
    ] {
        surface.graphics.placements.push(placement(source));
        assert_eq!(draw(&view, &surface, cx), first);
        surface.graphics.placements.pop();
        assert_eq!(draw(&view, &surface, cx), first);
    }
}
