// ABOUTME: Qualifies enlarged native Settings text at wide and minimum supported widths.
// ABOUTME: Uses real private configuration and emits bounded GPU captures for visual inspection.
use super::{GuiTarget, Running, fs};

#[test]
#[ignore = "requires active Windows desktop; captures only its private 24-point Settings window"]
fn native_settings_enlarged_text_keeps_its_row_inside_wide_and_narrow_windows() {
    let mut running = Running::prepared();
    let config = running
        .sandbox
        .dir
        .join("config/spaceport/config.local.toml");
    let source = "language = \"en-US\"\nsmooth_scroll = true\n[ui]\nsize = 24\n";
    fs::write(&config, source).unwrap();
    fs::write(
        running.sandbox.dir.join("uia_settings.ps1"),
        include_str!("../support/uia_settings.ps1"),
    )
    .unwrap();
    running.start_gui(GuiTarget::SettingsScaling);
    running.wait_tree("private Settings opener", |tree| {
        tree.contains("Button 'Settings'")
    });
    running.settings_action("Scale");
    let log = running.gui_log();
    assert!(
        log.contains("SETTINGS_SCALE_PASS ui=24 wide=1100x780 narrow=680x780"),
        "{log}"
    );
    for name in ["wide", "narrow"] {
        let prefix = format!("SETTINGS_SCALE_PNG {name} ");
        let captures: Vec<_> = log
            .lines()
            .filter(|line| line.starts_with(&prefix))
            .collect();
        assert_eq!(
            captures.len(),
            1,
            "native Settings must emit one {name} capture"
        );
        eprintln!("{}", captures[0]);
    }
    assert_eq!(
        fs::read_to_string(config).unwrap(),
        source,
        "native scaling changed private preferences"
    );
    eprintln!(
        "SETTINGS_SCALE native PASS: loaded 24-point UI, actual wide/narrow viewport and contained focused row; glyph inspection still required"
    );
}
