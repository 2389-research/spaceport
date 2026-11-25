use super::*;

/// The build script refuses files that do not parse; what it embeds is
/// every English file, and the table `fl!` searches is sorted.
#[test]
fn the_embedded_files_are_the_english_ones_and_the_table_is_sorted() {
    assert!(
        __private::FILES
            .iter()
            .any(|file| file.language == "en-US" && file.feature == "commands")
    );
    assert!(__private::FILES.iter().all(|file| file.language == "en-US"));
    assert!(
        __private::MESSAGES
            .windows(2)
            .all(|pair| pair[0].0 < pair[1].0)
    );
    assert_eq!(available(), vec![Locale::FALLBACK]);
}
