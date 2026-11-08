//! The `en-XA` leak test for every provider's tab: Settings' own page test
//! sees only the first.
#![allow(clippy::unwrap_used)]

use super::super::super::{Section, SettingsWindow};
use crate::cloud::CloudProvider;
use crate::i18n::leak_tests::{assert_translated, draw, under_en_xa};

/// What the tabs show that is not a message: provider names, as tabs and
/// as card titles, and the values fields show as their placeholders.
fn data() -> Vec<String> {
    let names = CloudProvider::ALL
        .iter()
        .map(|&provider| crate::cloud::name(provider));
    let data: Vec<String> = names
        .clone()
        .map(str::to_owned)
        .chain(names.map(str::to_uppercase))
        .collect();
    data
}

#[test]
fn every_provider_tab_shows_only_messages_and_provider_names() {
    under_en_xa("Cloud devices en-XA leak", |cx, recorder| {
        let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
        let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
        recorder.take();
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = SettingsWindow::new(weak, cx);
            // Unconfigured accounts read nothing from disk or the network.
            view.section = Section::CloudDevices;
            view.open_cloud_devices(cx);
            window.focus(&view.focus, cx);
            view
        });
        cx.simulate_resize(gpui::size(gpui::px(1200.), gpui::px(3000.)));
        let data = data();
        for &provider in CloudProvider::ALL {
            cx.update(|_, cx| {
                view.update(cx, |view, cx| {
                    view.cloud_tab = provider;
                    cx.notify();
                })
            });
            draw(cx);
            // Lines an earlier tab drew are not shaped again, so each tab is
            // checked by what it adds.
            assert_translated(
                &format!("Settings > Cloud devices > {provider:?}"),
                &recorder.take(),
                &data,
            );
        }
    });
}
