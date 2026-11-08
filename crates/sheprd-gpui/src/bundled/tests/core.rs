//! Core startup selects an independently installed executable on PATH.
//! A copy inside the app bundle cannot become its runtime by accident.
use super::*;

#[test]
fn installed_core_skips_the_app_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("Spaceport.app/Contents/MacOS");
    let installed = temp.path().join("installed");
    std::fs::create_dir_all(&app).unwrap();
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

#[test]
fn installed_core_can_share_a_standalone_bin_directory() {
    let temp = tempfile::tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let exe = bin.join("spaceport");
    let core = bin.join(sheprd_core_client::PROGRAM);
    helper(&exe);
    helper(&core);
    let path = std::env::join_paths([&bin]).unwrap();

    assert_eq!(
        locate_installed_in(sheprd_core_client::PROGRAM, Some(&exe), Some(&path)),
        Some(core.canonicalize().unwrap())
    );
}
