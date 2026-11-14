// ABOUTME: Checks that desktop labels name the owner of their data.
// ABOUTME: Preserves runtime versions and client names in English and pseudo-locales.
use super::*;
use sheprd_i18n::Pseudo;

#[test]
fn daemon_release_labels_identify_herdr() {
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        show(&locale);
        let labels = [
            "window-announcement",
            "window-announcement-preview",
            "whats-new-update-ready",
            "whats-new-title",
        ]
        .map(|id| {
            message(
                id,
                &[("runtime", "Herdr".into()), ("version", "9.8.7".into())],
            )
        });
        show(&Locale::FALLBACK);
        for label in labels {
            assert!(label.contains("Herdr"), "{locale:?}: {label}");
            assert!(label.contains("9.8.7"), "{locale:?}: {label}");
            assert!(
                !label.contains("Sheprd") && !label.contains("Spaceport"),
                "{label}"
            );
        }
    }
}

#[test]
fn client_fallback_labels_preserve_spaceport() {
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        show(&locale);
        let label = message(
            "settings-integrations-no-window",
            &[("app", "Spaceport".into())],
        );
        show(&Locale::FALLBACK);
        assert!(label.contains("Spaceport"), "{locale:?}: {label}");
        assert!(!label.contains("Sheprd"), "{label}");
    }
}

#[test]
fn shared_chat_rejection_uses_client_neutral_text() {
    show(&Locale::FALLBACK);
    let rejection = sheprd_chat_protocol::Rejection::Unsupported;
    let text = rejection.localize().replace(['\u{2068}', '\u{2069}'], "");
    assert_eq!(
        text,
        "this chat program is older than this client; start a new chat to use that"
    );
    assert_eq!(text, rejection.to_string());
}
