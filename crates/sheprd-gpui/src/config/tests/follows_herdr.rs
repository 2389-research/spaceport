use super::*;

/// `Follow Herdr` is asked of the side an appearance shows, so a pair can
/// follow Herdr in one appearance and name a theme in the other.
#[test]
fn a_theme_follows_herdr_by_the_side_shown() {
    for (value, light, dark) in [
        (FOLLOW_HERDR, true, true),
        (" Follow Herdr ", true, true),
        ("Nord", false, false),
        ("light:Follow Herdr,dark:Dracula", true, false),
        ("dark:Follow Herdr,light:Dracula", false, true),
        ("light:Follow Herdr,dark:Follow Herdr", true, true),
        // Not a valid pair, so not the sentinel either.
        ("light:Follow Herdr", false, false),
        ("follow herdr", false, false),
    ] {
        assert_eq!(ThemeName::follows_herdr(value, true), light, "{value}");
        assert_eq!(ThemeName::follows_herdr(value, false), dark, "{value}");
    }
}
