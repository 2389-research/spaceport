use super::*;

fn layout(text: &str) -> anyhow::Result<SidebarLayout> {
    Ok(SidebarLayout::from_daemon_config(&text.parse()?)?)
}

#[test]
fn configured_default_rows_match_typed_defaults() -> anyhow::Result<()> {
    let text = "[ui.sidebar.agents]\nrows = [[\"state_icon\", \"machine\", \"workspace\", \"tab\"], [\"agent\"]]\n\
                [ui.sidebar.spaces]\nrows = [[\"state_icon\", \"workspace\"], [\"branch\"]]\n";
    assert_eq!(layout(text)?, SidebarLayout::default());
    Ok(())
}

#[test]
fn scoped_rows_keep_custom_styles_and_agent_overrides() -> anyhow::Result<()> {
    let layout = layout(
        "[ui.sidebar.agents]\nrows = [[{ token = \"$model\", bold = true }]]\n\
         [ui.sidebar.agents.rows_by_agent]\nclaude = [[\"$summary\"]]\n\
         [ui.sidebar.spaces]\nrows = [[\"state_text\"]]\n",
    )?;
    assert_eq!(
        layout.agents.rows[0][0].token,
        "$model".parse::<AgentToken>()?
    );
    assert_eq!(
        layout.agents.rows_by_agent["claude"][0][0].token,
        "$summary".parse::<AgentToken>()?
    );
    assert_eq!(
        layout.spaces.rows[0][0].token,
        "state_text".parse::<SpaceToken>()?
    );
    assert!("terminal_title".parse::<SpaceToken>().is_err());
    assert!("$bad name".parse::<AgentToken>().is_err());
    Ok(())
}
