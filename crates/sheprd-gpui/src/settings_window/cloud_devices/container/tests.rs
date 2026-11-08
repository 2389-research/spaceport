#![allow(clippy::unwrap_used)]
use super::{
    super::super::{Section, SettingsWindow},
    CloudProvider, ContainerFields, Device, Overview, State,
};
use crate::{cloud::SavedDevice, container::Engine};
use gpui::{Entity, TestAppContext, VisualTestContext, px, size};

mod messages;

fn open(cx: &mut TestAppContext) -> (Entity<SettingsWindow>, &mut VisualTestContext) {
    let main = cx.add_window(crate::sidebar::layout_tests::fixture_window);
    let weak = cx.update(|cx| main.update(cx, |_, _, cx| cx.weak_entity()).unwrap());
    let (view, cx) = cx.add_window_view(|window, cx| {
        let view = SettingsWindow::new(weak, cx);
        window.focus(&view.focus, cx);
        view
    });
    cx.simulate_resize(size(px(960.), px(900.)));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.config.container = Default::default();
            view.select_section(Section::CloudDevices, window, cx);
            view.cloud_tab = CloudProvider::Container;
            cx.notify();
        })
    });
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

fn device(id: &str) -> SavedDevice {
    SavedDevice {
        provider: CloudProvider::Container,
        id: id.into(),
        label: id.into(),
        account: "docker".into(),
        machine: id.into(),
        session: "default".into(),
        enabled: true,
    }
}

fn set_overview(view: &Entity<SettingsWindow>, cx: &mut VisualTestContext, overview: Overview) {
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            view.container_card.as_mut().unwrap().overview = Some(overview);
            cx.notify();
        })
    });
}

#[gpui::test]
fn the_card_keeps_existing_device_controls_without_creation(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    draw(cx);
    assert!(cx.debug_bounds("cloud-container-card").is_some());
    assert!(cx.debug_bounds("cloud-container-create").is_none());
    assert!(cx.debug_bounds("cloud-container-adopt-0").is_none());
    view.read_with(cx, |view, cx| {
        let card = view.container_card.as_ref().unwrap();
        assert!(card.overview.is_none(), "tests never ask the real engine");
        assert_eq!(card.values(cx), ContainerFields::default());
    });
}

#[gpui::test]
fn saved_container_controls_do_not_stop_it_and_missing_engine_is_explained(
    cx: &mut TestAppContext,
) {
    let (view, cx) = open(cx);
    set_overview(
        &view,
        cx,
        Overview {
            engine: Ok((Engine::Docker, "29.4.0".into())),
            devices: vec![
                Device {
                    saved: device("herdr-calm-otter-1a"),
                    state: Some(State::Running),
                },
                Device {
                    saved: device("herdr-still-lake-2b"),
                    state: Some(State::Stopped),
                },
            ],
        },
    );
    draw(cx);
    assert!(cx.debug_bounds("cloud-container-stop-0").is_none());
    assert!(cx.debug_bounds("cloud-container-remove-1").is_some());
    assert!(cx.debug_bounds("cloud-container-adopt-0").is_none());
    set_overview(
        &view,
        cx,
        Overview {
            engine: Err("Neither docker nor podman was found.".into()),
            devices: Vec::new(),
        },
    );
    draw(cx);
    assert!(cx.debug_bounds("cloud-container-stop-0").is_none());
    assert!(cx.debug_bounds("cloud-container-unsaved").is_none());
}
