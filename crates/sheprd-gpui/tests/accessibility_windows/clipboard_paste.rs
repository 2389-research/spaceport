// ABOUTME: Requires real native paste, prompt bounds, cancellation, and recovery receipts.
// ABOUTME: Runs chat capture and standalone text acquisition in separate owned GUI fixtures.
use super::{GuiTarget, clipboard_broker};

fn check(target: GuiTarget, scenario: &str) {
    let output = clipboard_broker::application_check(target, scenario);
    for receipt in [
        "CLIPBOARD_APPLICATION unicode_crlf=true",
        "CLIPBOARD_APPLICATION exact_utf8_bytes=262144 accepted=true",
        "CLIPBOARD_APPLICATION rejected_utf8_bytes=262145 draft_preserved=true error=Prompt",
        "CLIPBOARD_APPLICATION rejected_utf8_bytes=2097153 draft_preserved=true error=Acquisition",
        "CLIPBOARD_APPLICATION cancel=Type observed_before_release=true reader_still_active=true",
        "CLIPBOARD_APPLICATION cancel=Type stale_absent=true reader_recovered=true",
        "CLIPBOARD_APPLICATION cancel=Blur observed_before_release=true reader_still_active=true",
        "CLIPBOARD_APPLICATION cancel=Blur stale_absent=true reader_recovered=true",
    ] {
        assert!(output.contains(receipt), "{output}");
    }
    assert!(
        output.contains(&format!(
            "CLIPBOARD_APPLICATION native PASS scenario={scenario}"
        )),
        "{output}"
    );
}

#[test]
#[ignore = "requires a disposable empty Windows clipboard and an active desktop; pastes only authored text"]
fn native_chat_composer_paste_limits_cancellation_and_reader_recovery() {
    check(GuiTarget::ChatClipboard, "Chat");
}

#[test]
#[ignore = "requires a disposable empty Windows clipboard and an active desktop; exercises standalone text acquisition"]
fn native_standalone_field_paste_limits_cancellation_and_reader_recovery() {
    check(GuiTarget::ClipboardField, "Field");
}
