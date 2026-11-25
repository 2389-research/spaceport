//! Every id the section list and the footer name is a message in English, so
//! none of them can go missing from a language.

use super::*;

#[test]
fn every_id_the_list_and_footer_names_is_a_message() {
    for id in MESSAGE_IDS {
        assert!(spaceport_i18n::takes(id).is_some(), "{id} names no message");
    }
    assert_eq!(
        spaceport_i18n::takes(UNSAVED),
        Some(&["drafts"][..]),
        "the footer fills the waiting drafts into {UNSAVED}"
    );
}
