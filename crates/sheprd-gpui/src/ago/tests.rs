use super::*;
use sheprd_i18n::{Locale, Pseudo};

#[test]
fn reads_in_the_largest_whole_unit() {
    for (seconds, expected) in [
        (0, ("ago-now", None)),
        (59, ("ago-now", None)),
        (60, ("ago-minutes", Some(1))),
        (59 * 60, ("ago-minutes", Some(59))),
        (3600, ("ago-hours", Some(1))),
        (23 * 3600, ("ago-hours", Some(23))),
        (86_400, ("ago-days", Some(1))),
        (2 * 86_400 - 1, ("ago-days", Some(1))),
        (3 * 86_400, ("ago-days", Some(3))),
        (21 * 86_400, ("ago-days", Some(21))),
    ] {
        assert_eq!(message(seconds), expected, "{seconds}");
    }
}

#[test]
fn the_window_isolates_the_count_and_translates_every_unit() {
    i18n::show(&Locale::FALLBACK);
    assert_eq!(text(120), "\u{2068}2\u{2069} min ago");
    i18n::show(&Pseudo::Accented.locale());
    for seconds in [0, 120, 7200, 86_400, 3 * 86_400] {
        let shown = text(seconds);
        assert!(sheprd_i18n::pseudo::is_marked(&shown), "{shown}");
    }
    i18n::show(&Locale::FALLBACK);
}
