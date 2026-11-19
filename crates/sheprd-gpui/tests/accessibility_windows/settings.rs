// ABOUTME: Qualifies native Settings keys, saved preferences, and the painted focus ring.
// ABOUTME: Scopes every action and image to the fixture's separate owned Settings window.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Read, Running, fs, thread};

impl Running {
    fn settings_log(&self) -> String {
        let mut bytes = Vec::new();
        fs::File::open(self.sandbox.dir.join("gui.log"))
            .unwrap()
            .take((CAPTURE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(
            bytes.len() <= CAPTURE_LIMIT,
            "native Settings log exceeds bound"
        );
        String::from_utf8(bytes).unwrap()
    }

    fn settings_action(&mut self, action: &str) {
        let script = self.sandbox.dir.join("uia_settings.ps1");
        let result = self.script_output(&script, "-Action", action, None, Duration::from_secs(35));
        if result.is_err() {
            eprintln!("{}", self.settings_log());
        }
        let output = result.unwrap();
        eprintln!("{output}");
        assert!(
            output.contains(&format!("SETTINGS_KEY action={action} native=true")),
            "{output}"
        );
    }

    fn settings_saved(&mut self, smooth: bool, layout: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let path = self.sandbox.dir.join("config/spaceport/config.local.toml");
        loop {
            let text = fs::read_to_string(&path).unwrap();
            assert!(
                text.len() <= 65_536,
                "private Settings config exceeds bound"
            );
            let value: toml::Value = toml::from_str(&text).unwrap();
            if value.get("smooth_scroll").and_then(toml::Value::as_bool) == Some(smooth)
                && value
                    .get("chat")
                    .and_then(|chat| chat.get("layout"))
                    .and_then(toml::Value::as_str)
                    == Some(layout)
            {
                assert_eq!(
                    value.get("language").and_then(toml::Value::as_str),
                    Some("en-US")
                );
                eprintln!("SETTINGS_SAVED smooth_scroll={smooth} layout={layout} language=en-US");
                return;
            }
            assert!(
                Instant::now() < deadline,
                "native Settings did not persist the requested preference"
            );
            assert!(
                self.gui.as_mut().unwrap().try_wait().unwrap().is_none(),
                "owned Settings GUI exited"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
}

#[test]
#[ignore = "requires an active Windows desktop; uses only private Settings preferences"]
fn native_settings_keyboard_persistence_and_painted_focus_ring() {
    let mut running = Running::prepared();
    fs::write(running.sandbox.dir.join("config/spaceport/config.local.toml"),
        "language = \"en-US\"\nsmooth_scroll = true\n[chat]\nlayout = \"timeline\"\n[features]\nacp_chats = true\n").unwrap();
    fs::write(
        running.sandbox.dir.join("uia_settings.ps1"),
        include_str!("../support/uia_settings.ps1"),
    )
    .unwrap();
    running.start_gui(GuiTarget::SettingsKeyboard);
    running.wait_tree("Settings opener", |tree| tree.contains("Button 'Settings'"));
    running.settings_action("OpenAndRing");
    let log = running.settings_log();
    eprintln!("{log}");
    assert!(log.contains("SETTINGS_RING_PASS "), "{log}");
    assert!(
        log.contains("SETTINGS_RING_PNG focused ") && log.contains("SETTINGS_RING_PNG unfocused "),
        "{log}"
    );
    for (action, smooth, layout) in [
        ("ToggleSpace", false, "timeline"),
        ("ToggleEnter", true, "timeline"),
        ("Bubbles", true, "bubbles"),
        ("Timeline", true, "timeline"),
        ("Dropdown", true, "timeline"),
    ] {
        running.settings_action(action);
        running.settings_saved(smooth, layout);
    }
    eprintln!(
        "SETTINGS native PASS: Tab/Shift-Tab, native Space/Enter switches and segments, dropdown return focus, private saved preferences, painted focus ring"
    );
}
