// ABOUTME: Qualifies rendered chat semantics and deliberate native permission answers.
// ABOUTME: Uses authored data and distinguishes queued intent from connected-agent delivery.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Read, Running, fs, thread};
use base64::Engine as _;

const FIXTURE: &str = include_str!("../support/native-chat-accessibility.json");

impl Running {
    fn chat_action(&mut self, action: &str) {
        let output = self
            .script_output(
                &self.sandbox.dir.join("uia_chat.ps1"),
                "-Action",
                action,
                None,
                Duration::from_secs(20),
            )
            .unwrap();
        assert!(
            output.contains(&format!("CHAT_KEY action={action} native=true")),
            "{output}"
        );
    }

    fn wait_chat_receipt(&mut self, expected: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let mut bytes = Vec::new();
            fs::File::open(self.sandbox.dir.join("gui.log"))
                .unwrap()
                .take((CAPTURE_LIMIT + 1) as u64)
                .read_to_end(&mut bytes)
                .unwrap();
            assert!(
                bytes.len() <= CAPTURE_LIMIT,
                "native chat log exceeds bound"
            );
            let log = String::from_utf8_lossy(&bytes);
            if log.contains(expected) {
                return;
            }
            assert!(
                self.gui.as_mut().unwrap().try_wait().unwrap().is_none(),
                "native chat exited:\n{log}"
            );
            assert!(
                Instant::now() < deadline,
                "missing native chat receipt {expected}:\n{log}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
}

#[test]
#[ignore = "requires an active Windows desktop; uses an authored chat without a daemon or agent"]
fn native_chat_content_controls_and_permission_keyboard_answers() {
    let mut running = Running::prepared();
    for (name, source) in [
        ("native-chat-accessibility.json", FIXTURE),
        ("uia_chat.ps1", include_str!("../support/uia_chat.ps1")),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    running.start_gui(GuiTarget::ChatAccessibility);
    running.wait_chat_receipt("RICH CHAT READY case=accessibility");
    let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let transcript = &fixture["transcript"];
    let tree = running.wait_tree("readable native chat and named controls", |tree| {
        [
            "Group 'You'",
            "Group 'Native fixture agent'",
            "Group 'Run: done'",
            "List 'PLAN'",
            "Group 'Native permission for Enter'",
            "Group 'Native permission for Space'",
            "Edit 'Message the agent' ",
            "Button 'Allow once'",
            "Button 'Reject once'",
            "Button 'Agent settings'",
            "Button 'Attach files'",
            "Button 'Send message'",
            "Button 'Show terminal'",
            "Button 'Change chat layout:",
        ]
        .iter()
        .all(|name| tree.contains(name))
            && transcript["entries"]
                .as_array()
                .unwrap()
                .iter()
                .take(2)
                .all(|entry| tree.contains(entry["kind"]["text"].as_str().unwrap()))
            && transcript["plan"]
                .as_array()
                .unwrap()
                .iter()
                .zip(["done", "running", "pending"])
                .all(|(step, status)| {
                    tree.lines().any(|line| {
                        line.trim_start().starts_with("ListItem '")
                            && line.contains(step["content"].as_str().unwrap())
                            && line.contains(status)
                    })
                })
    });
    assert!(
        !tree.contains("Native expanded tool output"),
        "collapsed output leaked"
    );
    running.chat_action("Settings");
    running.wait_chat_receipt("CHAT_ACCESSIBILITY menu=open");
    running.read("{ESC}");
    running.wait_chat_receipt("CHAT_ACCESSIBILITY menu=closed");
    running.chat_action("Tool");
    running.wait_tree("expanded tool text through native UIA", |tree| {
        tree.contains("Text 'Native expanded tool output'")
    });
    running.wait_chat_receipt("CHAT_ACCESSIBILITY tool=expanded");
    running.chat_action("Tool");
    running.wait_tree("folded tool text removed from native UIA", |tree| {
        !tree.contains("Native expanded tool output")
    });
    running.wait_chat_receipt("CHAT_ACCESSIBILITY tool=collapsed");
    running.chat_action("Allow");
    running.wait_chat_receipt("CHAT_ACCESSIBILITY answer=1 queued=true draft=preserved");
    running.wait_tree(
        "first permission resolved without removing the second",
        |tree| {
            !tree.contains("Group 'Native permission for Enter'")
                && tree.contains("Group 'Native permission for Space'")
        },
    );
    running.chat_action("Reject");
    running.wait_chat_receipt("CHAT_ACCESSIBILITY answer=2 queued=true draft=preserved");
    running.wait_tree("both native permission prompts resolved", |tree| {
        !tree.contains("Group 'Native permission for Enter'")
            && !tree.contains("Group 'Native permission for Space'")
    });
    let mut png = Vec::new();
    fs::File::open(running.sandbox.dir.join("native-chat.png"))
        .unwrap()
        .take(524_289)
        .read_to_end(&mut png)
        .unwrap();
    assert!(png.len() <= 524_288, "native chat PNG exceeds 512 KiB");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n") && png.ends_with(b"\0\0\0\0IEND\xaeB`\x82"));
    eprintln!(
        "CHAT_ACCESSIBILITY_PNG {}",
        base64::engine::general_purpose::STANDARD.encode(png)
    );
    eprintln!(
        "CHAT_ACCESSIBILITY native PASS: named content and controls, Tab, Enter, Space, queued answers and preserved draft"
    );
}
