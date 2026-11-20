// ABOUTME: Captures selected native tabs before and after actual keyboard focus.
// ABOUTME: Keeps theme, image backdrop, configuration, and terminal identity inside private fixtures.
use super::{Duration, GuiTarget, Running, fs};
use std::collections::BTreeSet;

#[test]
#[ignore = "requires Windows desktop and HERDR_TEST_BINARY; native PNGs still require contrast inspection"]
fn native_selected_tab_focus_has_bounded_light_dark_and_image_capture_pairs() {
    for (theme, name) in [("Nord", "dark"), ("Catppuccin Latte", "light")] {
        for backdrop in [false, true] {
            let mut running = Running::prepared();
            let image = running.sandbox.dir.join("tab-backdrop.png");
            image::RgbaImage::from_fn(2, 1, |x, _| {
                image::Rgba(if x == 0 {
                    [0, 0, 0, 255]
                } else {
                    [255, 255, 255, 255]
                })
            })
            .save(&image)
            .unwrap();
            let config = running
                .sandbox
                .dir
                .join("config/spaceport/config.local.toml");
            let mut source =
                format!("language = 'en-US'\ntheme = '{theme}'\ntab_layout = 'classic'\n");
            if backdrop {
                source.push_str(&format!(
                    "[backdrop]\nopacity = 0.75\nimage = {}\n",
                    serde_json::to_string(image.to_str().unwrap()).unwrap()
                ));
            }
            fs::write(&config, &source).unwrap();
            for (file, text) in [
                (
                    "uia_tab_contrast.ps1",
                    include_str!("../support/uia_tab_contrast.ps1"),
                ),
                (
                    "native_tab_capture.cs",
                    include_str!("../support/native_tab_capture.cs"),
                ),
            ] {
                fs::write(running.sandbox.dir.join(file), text).unwrap();
            }
            running.start_daemon();
            running.start_gui(GuiTarget::Socket);
            running.wait_tree("private tab and terminal", |tree| {
                tree.contains("TabItem '1' [selected=True]")
                    && tree.contains("Document 'Terminal' [Value, Text]")
            });
            let tabs = running.daemon_result(&["tab", "list"]);
            let workspace = tabs["tabs"][0]["workspace_id"].as_str().unwrap();
            let pane_ids = |value: serde_json::Value| {
                value["panes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|pane| pane["pane_id"].as_str().unwrap().to_owned())
                    .collect::<BTreeSet<_>>()
            };
            let before =
                pane_ids(running.daemon_result(&["pane", "list", "--workspace", workspace]));
            assert_eq!(before.len(), 1, "tab capture must use one private terminal");
            let capture = running.sandbox.dir.join("tab-contrast.log");
            let mut command = running.powershell(
                &running.sandbox.dir.join("uia_tab_contrast.ps1"),
                "tab-contrast.log",
            );
            if backdrop {
                command.arg("-ImageBackdrop");
            }
            let output = running
                .command_output(command, &capture, Duration::from_secs(30))
                .unwrap();
            assert!(output.contains("TAB_RING_CAPTURE native=true selected=true focused=true stable_bounds=true released=true"), "{output}");
            let mode = if backdrop { "image" } else { "opaque" };
            for phase in ["unfocused", "focused"] {
                let prefix = format!("TAB_RING_PNG {phase} ");
                let images = output
                    .lines()
                    .filter(|line| line.starts_with(&prefix))
                    .collect::<Vec<_>>();
                assert_eq!(
                    images.len(),
                    1,
                    "expected one {name}/{mode}/{phase} native capture"
                );
                eprintln!(
                    "TAB_RING_PNG {name}-{mode}-{phase} {}",
                    images[0].strip_prefix(&prefix).unwrap()
                );
            }
            for line in output
                .lines()
                .filter(|line| line.starts_with("TAB_RING_") && !line.starts_with("TAB_RING_PNG"))
            {
                eprintln!("{name}-{mode}: {line}");
            }
            assert_eq!(
                pane_ids(running.daemon_result(&["pane", "list", "--workspace", workspace])),
                before
            );
            assert_eq!(fs::read_to_string(config).unwrap(), source);
        }
    }
}
