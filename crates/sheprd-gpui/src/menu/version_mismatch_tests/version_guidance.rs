// ABOUTME: Verifies the client-update notice preserves the Spaceport brand.
// ABOUTME: Checks version and host data in English and both pseudo-locales.
use super::*;
use sheprd_i18n::{Locale, Pseudo};

#[test]
fn client_update_notice_preserves_product_and_host_data() {
    let notice = VersionNotice {
        host: Some("build box".into()),
        mismatch: VersionMismatch::ClientOutdated {
            server_version: Some("9.8.7".into()),
        },
    };
    for locale in [
        Locale::FALLBACK,
        Pseudo::Accented.locale(),
        Pseudo::Bidi.locale(),
    ] {
        crate::i18n::show(&locale);
        let title = notice.title();
        let body = notice.body();
        crate::i18n::show(&Locale::FALLBACK);
        assert!(title.contains("Spaceport"), "{locale:?}: {title}");
        assert!(!title.contains("Sheprd"), "{locale:?}: {title}");
        assert!(body.contains("build box"), "{locale:?}: {body}");
        assert!(body.contains("9.8.7"), "{locale:?}: {body}");
    }
}
