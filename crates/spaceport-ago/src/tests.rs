use super::*;

#[test]
fn reads_in_the_largest_whole_unit() {
    for (seconds, expected) in [
        (0, Elapsed::Now),
        (59, Elapsed::Now),
        (60, Elapsed::Minutes(1)),
        (59 * 60, Elapsed::Minutes(59)),
        (3_600, Elapsed::Hours(1)),
        (23 * 3_600, Elapsed::Hours(23)),
        (86_400, Elapsed::Days(1)),
        (2 * 86_400 - 1, Elapsed::Days(1)),
        (3 * 86_400, Elapsed::Days(3)),
        (21 * 86_400, Elapsed::Days(21)),
    ] {
        assert_eq!(elapsed(seconds), expected, "{seconds}");
    }
}
