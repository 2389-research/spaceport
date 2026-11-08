use super::*;

#[test]
fn launcher_targets_only_the_spaceport_gui() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = dir.path().join("herdr-gpui.exe");
    let current = dir.path().join("spaceport.exe");
    std::fs::write(&legacy, b"legacy").unwrap();
    let launcher = dir.path().join("spaceport-launcher.exe");
    assert_eq!(gui_path(&launcher).unwrap(), current);
    std::fs::write(&current, b"current").unwrap();
    assert_eq!(gui_path(&launcher).unwrap(), current);
}
