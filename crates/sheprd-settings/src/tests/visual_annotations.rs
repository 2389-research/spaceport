use super::*;

#[test]
fn obsolete_page_annotation_binding_is_ignored() -> anyhow::Result<()> {
    let config = Config::parse("[keybindings]\ntoggle_annotate = 'cmd-shift-a'")?;
    assert_eq!(config.unknown_keys, ["keybindings.toggle_annotate"]);
    Ok(())
}
