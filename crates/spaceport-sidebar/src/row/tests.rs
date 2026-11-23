use super::*;
use core::prelude::v1::test;
use gpui::{Bounds, Pixels, point, px, size};
use spaceport_settings::Weight;

fn gutter() -> Bounds<Pixels> {
    Bounds::new(point(px(0.), px(244.)), size(px(231.), px(40.)))
}

#[test]
fn child_gutter_lines_land_on_whole_device_pixels() {
    for scale in [1., 2., 3.] {
        let row = gutter();
        let device = |value: Pixels| f32::from(value) * scale;
        let whole = |value: Pixels| (device(value) - device(value).round()).abs() < 0.001;
        for tree in [RowTree::Child, RowTree::LastChild] {
            let middle = row.top() + px(4. + line_height(&font()) / 2.);
            let [trunk, tick] = tree_lines(row, tree, middle, scale, false);
            // Both lines carry the same weight and start on the device grid, so
            // neither is drawn thinner or blurrier than the other.
            assert!(
                (trunk.size.width - tick.size.height).abs() < px(0.01),
                "{scale}"
            );
            assert!(
                (device(trunk.size.width) - scale.round().max(1.)).abs() < 0.01,
                "{scale}"
            );
            for edge in [trunk.left(), trunk.top(), tick.left(), tick.top()] {
                assert!(whole(edge), "{scale}: {edge:?}");
            }
            // The trunk hugs the gutter's leading edge, the tick crosses to the
            // dot at its far edge; neither strays into the label beyond.
            assert_eq!(trunk.left(), tick.left(), "{scale}");
            assert_eq!(trunk.left(), row.left(), "{scale}");
            assert_eq!(tick.right(), row.right(), "{scale}");
            // The tick meets the status dot's middle row.
            assert!(
                (tick.center().y - middle).abs() <= px(1. / scale),
                "{scale}"
            );
            // Only a row with a sibling below carries the trunk to the bottom.
            match tree {
                RowTree::Child => assert_eq!(trunk.bottom(), row.bottom(), "{scale}"),
                _ => assert_eq!(trunk.bottom(), tick.bottom(), "{scale}"),
            }
            assert_eq!(trunk.top(), row.top(), "{scale}");
        }
    }
}

#[test]
fn a_right_to_left_gutter_mirrors_its_trunk_and_tick() {
    let row = gutter();
    let middle = row.top() + px(24.);
    for tree in [RowTree::Child, RowTree::LastChild] {
        let [trunk, tick] = tree_lines(row, tree, middle, 2., true);
        // The leading edge is the gutter's right one, so the trunk runs down
        // it and the tick reaches back to the far edge.
        assert_eq!(trunk.right(), row.right(), "{tree:?}");
        assert_eq!(tick.right(), row.right(), "{tree:?}");
        assert_eq!(tick.left(), row.left(), "{tree:?}");
        assert_eq!(trunk.top(), row.top(), "{tree:?}");
    }
}

fn font() -> FontConfig {
    FontConfig {
        family: "Menlo".into(),
        size: 12.,
        fallbacks: None,
        line_height_multiple: None,
        weight: Weight::NORMAL,
    }
}
