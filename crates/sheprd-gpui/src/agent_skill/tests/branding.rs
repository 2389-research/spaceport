use super::*;

#[test]
fn spaceport_skill_owns_only_its_new_name_and_marker() {
    assert!(managed("<!-- spaceport-managed-skill v1: Spaceport -->"));
    assert!(!managed("<!-- herdr-gpui-managed-skill v1: Sheprd -->"));
    assert_eq!(NAME, "spaceport");
    assert_eq!(command(None), "spaceport");

    let home = tempfile::tempdir().unwrap();
    let root = home.path().join(".claude");
    std::fs::create_dir_all(&root).unwrap();
    let upstream = file(&root, "herdr-gpui-browser");
    std::fs::create_dir_all(upstream.parent().unwrap()).unwrap();
    let original = "<!-- herdr-gpui-managed-skill v1: personal copy -->\nprivate notes";
    std::fs::write(&upstream, original).unwrap();
    install(home.path(), SKILL).unwrap();
    remove(home.path()).unwrap();
    assert_eq!(std::fs::read_to_string(upstream).unwrap(), original);
}

#[test]
fn commands_are_substituted_once_even_when_the_install_path_ends_in_the_command() {
    assert_eq!(
        substitute_command(
            "spaceport open .\nspaceport open .",
            "spaceport",
            "/opt/bin/spaceport"
        ),
        "/opt/bin/spaceport open .\n/opt/bin/spaceport open ."
    );
}
