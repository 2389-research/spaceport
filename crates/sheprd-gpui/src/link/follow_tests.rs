use super::*;
use crate::endpoint::LOCAL;

#[gpui::test]
fn a_device_link_waits_until_the_saved_devices_are_read(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        assert!(!view.read(cx).catalog.loaded());
        defer(
            Link::Device {
                id: "ssh:later".into(),
            },
            cx,
        );
        view.update(cx, |view, cx| poll(view, window, cx));
        assert!(cx.global::<Pending>().0.is_some());
        defer(Link::Device { id: LOCAL.into() }, cx);
        view.update(cx, |view, cx| poll(view, window, cx));
        assert!(cx.global::<Pending>().0.is_none());
    });
}

#[gpui::test]
fn a_notification_from_before_startup_is_not_replayed(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        defer(
            Link::Notification {
                tag: "herdr:local:old".into(),
            },
            cx,
        );
        view.update(cx, |view, cx| poll(view, window, cx));
        assert!(cx.global::<Pending>().0.is_none());
    });
}

const REFUSALS: [Refusal; 4] = [
    Refusal::Closed,
    Refusal::DialogOpen,
    Refusal::NoDevice(String::new()),
    Refusal::NotificationGone,
];

#[test]
fn a_refused_link_answers_the_command_line_in_english() {
    let message = |refusal: Refusal| match refusal.response() {
        Response::Error { message, .. } => message,
        other => panic!("not an error: {other:?}"),
    };
    crate::i18n::leak_tests::under_en_xa("link refusals in English", |_, _| {
        assert_eq!(message(Refusal::NoDevice("nas".into())), "No device nas");
        assert_eq!(
            REFUSALS.map(message),
            [
                "The window closed",
                "A dialog is open in the window; close it first",
                "No device ",
                "That notification is no longer known",
            ]
        );
    });
}

#[test]
fn a_refused_link_tells_the_window_in_messages() {
    crate::i18n::leak_tests::under_en_xa("link refusals en-XA leak", |_, _| {
        for refusal in REFUSALS {
            let shown = crate::i18n::fl!("link-open-failed", reason = refusal.shown());
            assert!(
                sheprd_i18n::pseudo::is_marked(&shown),
                "{refusal:?}: {shown}"
            );
        }
    });
}
