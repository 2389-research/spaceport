// ABOUTME: Checks the focus ring against each tab layout's actual fills.
// ABOUTME: Covers builtin themes and custom accents that blend into chrome.
#![allow(clippy::unwrap_used)]
use crate::{
    browser::{GroupIds, Slot},
    config::{TabLayoutMode, Theme},
    contrast::{Contrast, ratio},
    sidebar::{Indicators, layout_tests::fixture_window},
    tab_layout::TabState,
    window::chrome_focus::Region,
};

#[gpui::test]
fn selected_tabs_keep_a_visible_focus_ring(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.chrome_region = Some(Region::TitleBar);
        for name in Theme::BUILTIN_NAMES {
            for contrast in [Contrast::Standard, Contrast::High] {
                for blend in [false, true] {
                    view.theme = Theme::builtin(name).unwrap().with_contrast(contrast);
                    if blend { view.theme.accent = Some(view.theme.background); }
                    for &layout in TabLayoutMode::ALL {
                        view.config.tab_layout = layout;
                        let slot = Slot { id: GroupIds::default().next(), index: 0 };
                        let indicators = Indicators::new(None, Default::default(), false, false, &view.theme);
                        let ring = view.tab_context(slot, indicators).ring.unwrap();
                        for selected in [false, true] {
                            for active in [false, true] {
                                let (fill, _) = view.tab_layout().colors(TabState { selected, active }, &view.theme);
                                assert!(ratio(ring, fill) >= contrast.mark_ratio(),
                                    "{name} {contrast:?} {layout:?} blend={blend} selected={selected} active={active}: {ring:06x} on {fill:06x}");
                            }
                        }
                    }
                }
            }
        }
    });
}
