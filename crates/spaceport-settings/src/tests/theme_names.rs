//! Theme search filters every name the library holds, whatever script a token
//! is written in, and finds nothing for a word no name carries.

use crate::theme::filter_names;

#[test]
fn the_whole_catalog_and_unicode_tokens_are_not_capped() {
    let names: Vec<_> = (0..400)
        .map(|index| format!("Catalog {index:03}"))
        .collect();
    assert_eq!(
        filter_names(names.iter().map(String::as_str), "").len(),
        400
    );
    assert_eq!(
        filter_names(names.iter().map(String::as_str), "399 CATALOG"),
        ["Catalog 399"]
    );
    assert_eq!(
        filter_names(["Été 東京 Night", "Other"], "東京 ÉTÉ"),
        ["Été 東京 Night"]
    );
    assert!(filter_names(["Nord"], "not found").is_empty());
}
