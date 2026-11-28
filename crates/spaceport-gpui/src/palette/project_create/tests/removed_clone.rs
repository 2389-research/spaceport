use super::*;

#[test]
fn project_palette_does_not_offer_repository_clones() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let roots = [root.path().to_string_lossy().into_owned()];
    for input in [
        "https://github.com/owner/project.git",
        "ssh://git@host/owner/project",
        "git@host:owner/project.git",
    ] {
        assert_eq!(plan(&roots, input)?, None, "{input}");
    }
    assert_eq!(
        plan(&roots, "new-folder")?.map(|item| item.label),
        Some("new-folder".into())
    );
    Ok(())
}
