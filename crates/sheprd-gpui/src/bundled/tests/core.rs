//! Core startup selects an independently installed executable on PATH.
//! A copy beside the GUI cannot become its runtime by accident.
use super::*;

#[test]
fn installed_core_skips_the_app_directory() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("app");
    let installed = temp.path().join("installed");
    std::fs::create_dir(&app).unwrap();
    std::fs::create_dir(&installed).unwrap();
    let exe = app.join("spaceport");
    helper(&exe);
    let beside = app.join(sheprd_core_client::PROGRAM);
    let selected = installed.join(sheprd_core_client::PROGRAM);
    helper(&beside);
    helper(&selected);
    let path = std::env::join_paths([&app, &installed]).unwrap();
    assert_eq!(
        locate_installed_in(sheprd_core_client::PROGRAM, Some(&exe), Some(&path)),
        Some(selected.canonicalize().unwrap())
    );
    let only_app = std::env::join_paths([&app]).unwrap();
    assert_eq!(
        locate_installed_in(sheprd_core_client::PROGRAM, Some(&exe), Some(&only_app)),
        None
    );
}
