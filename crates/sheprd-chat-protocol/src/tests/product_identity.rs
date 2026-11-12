// ABOUTME: Keeps shared chat refusals independent of a desktop product name.
// ABOUTME: Verifies the existing unsupported wire value and its display text.
use super::*;

#[test]
fn unsupported_wire_value_is_stable_and_display_names_the_client() {
    let rejection = Rejection::Unsupported;
    assert_eq!(
        serde_json::to_string(&rejection).unwrap(),
        "\"unsupported\""
    );
    assert_eq!(
        serde_json::from_str::<Rejection>("\"unsupported\"").unwrap(),
        rejection
    );
    assert_eq!(
        rejection.to_string(),
        "this chat program is older than this client; start a new chat to use that"
    );
}
