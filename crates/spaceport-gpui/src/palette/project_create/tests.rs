use super::*;

mod removed_clone;

#[test]
fn accepts_plain_folder_names() {
    assert_eq!(parse_name("my-project").as_deref(), Some("my-project"));
    assert_eq!(
        parse_name("  spaced name  ").as_deref(),
        Some("spaced name")
    );
    assert_eq!(parse_name("v2.0").as_deref(), Some("v2.0"));
}

#[test]
fn refuses_names_that_are_not_one_segment() {
    for bad in ["", "   ", ".", "..", ".hidden", "a/b", "a\\b", "a:b"] {
        assert_eq!(parse_name(bad), None, "should refuse {bad:?}");
    }
}

#[test]
fn plans_a_folder_in_the_first_root() -> anyhow::Result<()> {
    // An absolute root expands to itself; HOME is unset on Windows runners.
    let dir = tempfile::tempdir()?;
    let root = dir.path().to_string_lossy().into_owned();
    let Some(plan) = plan(std::slice::from_ref(&root), "brand-new")? else {
        anyhow::bail!("expected a plan for a plain name");
    };
    assert_eq!(plan.path, PathBuf::from(&root).join("brand-new"));
    assert_eq!(plan.label, "brand-new");
    Ok(())
}

#[test]
fn plans_nothing_without_a_root() -> anyhow::Result<()> {
    assert_eq!(plan(&[], "anything")?, None);
    Ok(())
}

#[test]
fn a_second_project_of_the_same_name_is_refused() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let project = NewProject {
        path: root.path().join("taken"),
        label: "taken".into(),
    };
    fs::create_dir(&project.path)?;
    assert!(matches!(
        materialise(&project),
        Err(crate::Error::Palette(Error::ProjectExists))
    ));
    Ok(())
}

#[test]
fn a_new_folder_opens_at_its_resolved_path() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let project = NewProject {
        path: root.path().join("fresh"),
        label: "fresh".into(),
    };
    let opened = materialise(&project)?;
    assert_eq!(
        opened.path,
        crate::palette::canonical::canonicalize(root.path())?.join("fresh")
    );
    assert_eq!(opened.label, "fresh");
    assert!(opened.path.is_dir());
    Ok(())
}
