#![allow(clippy::unwrap_used)]
use super::*;
use spaceport_backend::TerminalId;

#[test]
fn a_mirror_tab_saves_that_it_only_watches() {
    let mirror = Location::CoreTerminal {
        panes: Panes::single(TerminalId::try_from("t4".to_owned()).unwrap()),
        observe: true,
    };
    let saved = serde_json::to_string(&mirror).unwrap();
    assert!(saved.contains("\"observe\":true"), "{saved}");
    assert_eq!(serde_json::from_str::<Location>(&saved).unwrap(), mirror);
    // A tab that types saves no flag at all.
    let typing = Location::CoreTerminal {
        panes: Panes::single(TerminalId::try_from("t4".to_owned()).unwrap()),
        observe: false,
    };
    assert!(!serde_json::to_string(&typing).unwrap().contains("observe"));
}

#[test]
fn a_tab_saved_before_splits_reads_back_as_one_pane() {
    let saved = r#"{"kind":"core_terminal","terminal":"t4"}"#;
    let location: Location = serde_json::from_str(saved).unwrap();
    assert_eq!(
        location,
        Location::CoreTerminal {
            panes: Panes::single(TerminalId::try_from("t4".to_owned()).unwrap()),
            observe: false,
        }
    );
    let again = serde_json::to_string(&location).unwrap();
    assert_eq!(serde_json::from_str::<Location>(&again).unwrap(), location);
}
