// ABOUTME: Checks real version-refusal diagnostics across display locales.
// ABOUTME: Preserves product/version data and the daemon-versus-app update choice.
use super::*;
use sheprd_client::{Error, protocol::endpoint::ENDPOINT_PROTOCOL_GENERATION};

#[test]
fn newer_runtime_errors_preserve_spaceport_and_reported_version() {
    let newer = ENDPOINT_PROTOCOL_GENERATION + 1;
    for error in [
        Error::BridgeIncompatible {
            generation: Some(newer),
            version: Some("9.8.7".into()),
        },
        Error::BridgeIncompatible {
            generation: Some(newer),
            version: None,
        },
        Error::EndpointGeneration {
            generation: newer,
            server_version: "9.8.7".into(),
        },
    ] {
        for locale in [
            Locale::FALLBACK,
            Pseudo::Accented.locale(),
            Pseudo::Bidi.locale(),
        ] {
            crate::i18n::show(&locale);
            let shown = error.localize();
            crate::i18n::show(&Locale::FALLBACK);
            assert!(shown.contains("Spaceport"), "{locale:?}: {shown}");
            assert!(!shown.contains("Sheprd"), "{locale:?}: {shown}");
            let version = match &error {
                Error::BridgeIncompatible { version, .. } => version.as_deref(),
                Error::EndpointGeneration { server_version, .. } => Some(server_version.as_str()),
                _ => unreachable!("only version-refusal fixtures"),
            };
            if let Some(version) = version {
                assert!(shown.contains(version), "{locale:?}: {shown}");
            }
        }
    }
}

#[test]
fn older_or_equal_runtime_keeps_the_herdr_update_command() {
    for generation in [0, ENDPOINT_PROTOCOL_GENERATION] {
        for error in [
            Error::BridgeIncompatible {
                generation: Some(generation),
                version: None,
            },
            Error::EndpointGeneration {
                generation,
                server_version: "0.8.2".into(),
            },
        ] {
            crate::i18n::show(&Locale::FALLBACK);
            let shown = error.localize().replace(ISOLATION, "");
            assert_eq!(shown, error.to_string());
            assert!(
                shown.contains("run `herdr update` and reconnect"),
                "{shown}"
            );
            assert!(!shown.contains("update Spaceport"), "{shown}");
        }
    }
}
