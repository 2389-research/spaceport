use super::*;

/// A test switching the locale switches only its own thread's catalog, so
/// another test reading text is not changed under it.
#[test]
fn a_switched_locale_is_the_switching_threads_alone() {
    let accented = std::thread::spawn(|| {
        set_locale(&Pseudo::Accented.locale());
        fl!("palette-filter-all")
    })
    .join()
    .unwrap();

    assert!(pseudo::is_marked(&accented), "{accented}");
    assert_eq!(shown_locale(), Some(Locale::FALLBACK));
    assert_eq!(fl!("palette-filter-all"), "All");
}

/// The locale shown is the one a test switched to, until it switches again.
#[test]
fn set_locale_reports_the_locale_shown() {
    let bidi = Pseudo::Bidi.locale();
    assert!(set_locale(&bidi));
    assert_eq!(shown_locale(), Some(bidi));
    assert_eq!(fl!("palette-filter-all"), "\u{202E}All\u{202C}");
}
