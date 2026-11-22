use super::*;
#[test]
fn sidebar_counts_exclude_pull_request_mode() -> anyhow::Result<()> {
    Config::parse(DEFAULT_CONFIG)?;
    let error = Config::parse("[sidebar]\nchange_counts = \"pr\"").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("sidebar.change_counts was removed")
    );
    Ok(())
}
