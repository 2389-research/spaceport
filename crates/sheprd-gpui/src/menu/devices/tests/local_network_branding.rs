// ABOUTME: Checks Spaceport's local-network guidance through the real formatter.
// ABOUTME: Preserves product and host data in English and both pseudo-locales.
use super::super::local_network;
use super::{Step, fixture_window, open_form, setup};
use crate::i18n::{Localize, show};
use sheprd_client::SshFailure;
use sheprd_i18n::{Locale, Pseudo};

#[test]
fn local_network_denial_names_spaceport_in_logs_and_locales() {
    let failure = SshFailure::LocalNetworkDenied;
    assert!(
        failure
            .to_string()
            .starts_with("macOS denied Spaceport access")
    );
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        show(&locale);
        let text = failure.localize();
        assert_eq!(
            text.matches("\u{2068}Spaceport\u{2069}").count(),
            2,
            "{text}"
        );
        assert!(!text.contains("Sheprd"), "{text}");
    }
    show(&Locale::FALLBACK);
}

#[test]
fn local_network_wait_names_spaceport_and_preserves_the_host() {
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        show(&locale);
        let text = local_network::waiting_status("dev@198.51.100.7");
        assert_eq!(
            text.matches("\u{2068}Spaceport\u{2069}").count(),
            3,
            "{text}"
        );
        assert!(text.contains("\u{2068}dev@198.51.100.7\u{2069}"), "{text}");
        assert!(!text.contains("Sheprd"), "{text}");
    }
    show(&Locale::FALLBACK);
}

#[gpui::test]
fn local_network_refusal_uses_localized_guidance(cx: &mut gpui::TestAppContext) {
    show(&Pseudo::Accented.locale());
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        let request = setup::Request::new("dev@198.51.100.7", "", "").unwrap();
        open_form(view, Step::AwaitingLocalNetwork(request), cx);
        view.device_probed(
            Ok((
                setup::Claim::fixture("local-network-branding"),
                Ok(sheprd_client::HostProbe::SshFailed(
                    SshFailure::LocalNetworkDenied,
                )),
            )),
            cx,
        );
    });
    let text = view.read_with(cx, |view, _| {
        view.menu
            .device_setup
            .as_ref()
            .unwrap()
            .error
            .clone()
            .unwrap()
    });
    show(&Locale::FALLBACK);
    assert!(sheprd_i18n::pseudo::is_marked(&text), "{text}");
    assert!(text.contains("\u{2068}Spaceport\u{2069}"), "{text}");
}
