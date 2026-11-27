//! The activity bar's panel is remembered with the rest of the chrome.
use super::*;
use crate::sidebar::ActivityPanel;

#[core::prelude::v1::test]
fn the_activity_panel_round_trips_and_an_unknown_one_is_forgotten() {
    let directory = TestDirectory::new();
    let path = directory.0.join("preferences.json");
    let chrome = Chrome {
        activity_panel: Some(ActivityPanel::Workspaces),
        sidebar_width: Some(250.),
        ..Chrome::default()
    };
    write_chrome(&path, chrome.clone()).unwrap();
    assert_eq!(read_chrome(&path).unwrap(), chrome);
    // Files from earlier builds, and panels a later build added, read as none.
    for stored in [
        "",
        r#", "activity_panel": "dashboard""#,
        r#", "activity_panel": 3"#,
    ] {
        fs::write(&path, format!(r#"{{"sidebar_width_px": 250{stored}}}"#)).unwrap();
        let read = read_chrome(&path).unwrap();
        assert_eq!(read.activity_panel, None, "{stored}");
        assert_eq!(read.sidebar_width, Some(250.), "{stored}");
    }
}

#[core::prelude::v1::test]
fn a_saved_choice_restores_as_workspaces_without_writing_on_read() {
    let directory = TestDirectory::new();
    let path = directory.0.join("preferences.json");
    for name in ["overview", "workspaces"] {
        let bytes = format!(r#"{{"sidebar_width_px":250,"activity_panel":"{name}"}}"#);
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            read_chrome(&path).unwrap().activity_panel,
            Some(ActivityPanel::Workspaces),
            "{name}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
    }
    // Sections earlier builds had are forgotten like any unknown one.
    for name in ["issues", "automations", "usage"] {
        fs::write(&path, format!(r#"{{"activity_panel":"{name}"}}"#)).unwrap();
        assert_eq!(read_chrome(&path).unwrap().activity_panel, None, "{name}");
    }
}
