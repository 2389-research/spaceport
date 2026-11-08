use super::*;

#[test]
fn a_terminal_tab_shows_its_programs_title_and_otherwise_terminal() {
    let terminal = Location::CoreTerminal {
        panes: crate::core_terminals::panes::Panes::single(
            sheprd_backend::TerminalId::try_from("t1".to_owned()).unwrap(),
        ),
        observe: false,
    };
    let mut store = Store::default();
    let id = store
        .open(Scope::endpoint("local"), "w", Some(terminal), None)
        .unwrap();
    let drawn = crate::i18n::message("browser-tab-terminal", &[]);
    assert_eq!(store.get(id).unwrap().label(), drawn);
    store.visited(id, None, Some("vim notes.md"));
    assert_eq!(store.get(id).unwrap().label(), "vim notes.md");
}
