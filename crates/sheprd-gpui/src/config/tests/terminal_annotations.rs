use super::*;

#[test]
fn obsolete_terminal_annotation_binding_is_ignored() -> anyhow::Result<()> {
    let config = Config::parse("[keybindings]\nannotate_selection = 'cmd-shift-a'")?;
    assert_eq!(config.unknown_keys, ["keybindings.annotate_selection"]);
    Ok(())
}
