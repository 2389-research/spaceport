// ABOUTME: Checks readable connection and chat configuration diagnostics.
// ABOUTME: Keeps host labels, reported reasons, and pseudo-localization intact.
use super::*;
use crate::i18n::fl;

#[test]
fn activation_notices_describe_terminal_readiness_without_wire_terms() {
    for locale in [Locale::FALLBACK, Pseudo::Accented.locale()] {
        crate::i18n::show(&locale);
        let messages = [
            fl!("endpoint-activation-failed", device = "build-box"),
            fl!(
                "endpoint-activation-failed-because",
                device = "build-box",
                reason = "connection lost"
            ),
            fl!("state-activation-invalid"),
            fl!("close-modal-batch-surface-timeout"),
        ];
        crate::i18n::show(&Locale::FALLBACK);
        assert!(messages[0].contains("build-box"));
        assert!(messages[1].contains("build-box"));
        assert!(messages[1].contains("connection lost"));
        for message in messages {
            if locale == Locale::FALLBACK {
                assert!(message.to_lowercase().contains("terminal"), "{message}");
                for word in ["surface", "barrier", "acknowledgement", "endpoint"] {
                    assert!(!message.to_lowercase().contains(word), "{word}: {message}");
                }
            } else {
                assert!(sheprd_i18n::pseudo::is_marked(&message), "{message}");
            }
        }
    }
}

#[test]
fn chat_mirror_and_unsupported_connection_errors_avoid_wire_terms() {
    assert_no_wire_words(&[crate::config::Error::InvalidChatMirror]);
    assert_no_wire_words(&[crate::endpoint::Error::SurfaceSwitchingUnsupported]);
}
