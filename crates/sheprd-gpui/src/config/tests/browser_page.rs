use super::*;

#[test]
fn obsolete_browser_page_preference_and_binding_are_ignored() -> anyhow::Result<()> {
    let config = Config::parse(
        "open_links_in = 'browser-tab'\n[keybindings]\nnew_browser_tab = 'cmd-shift-b'",
    )?;
    assert_eq!(
        config.unknown_keys,
        ["keybindings.new_browser_tab", "open_links_in"]
    );
    Ok(())
}
