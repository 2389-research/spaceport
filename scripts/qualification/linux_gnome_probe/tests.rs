// ABOUTME: Checks bounded native receipt extraction and the actual verdict consumer.
// ABOUTME: Covers noisy teardown logs and zero-test success without mocking the app.
use super::*;

#[test]
fn native_receipts_survive_chatty_cleanup_and_are_not_duplicated() {
    let mut log = format!(
        "{}\nLINUX_NATIVE case=new-tab native_evdev=true\n",
        receipt::NATIVE_PASS
    );
    for _ in 0..80 {
        log.push_str("daemon cleanup line\n");
    }
    log.push_str(receipt::NATIVE_PASS);
    let exported = native_receipts(&log).collect::<Vec<_>>();
    assert_eq!(
        exported
            .iter()
            .filter(|line| **line == receipt::NATIVE_PASS)
            .count(),
        1
    );
    assert!(exported.contains(&"LINUX_NATIVE case=new-tab native_evdev=true"));
    assert!(!exported.iter().any(|line| line.starts_with("daemon")));
}

#[test]
fn zero_tests_and_embedded_verdict_text_cannot_qualify_native_delivery() {
    for log in [
        String::new(),
        "test result: ok. 0 passed; 0 failed".to_owned(),
        format!("diagnostic quoted: {}", receipt::NATIVE_PASS),
    ] {
        assert!(require_native_verdict(&log).is_err());
    }
    assert!(require_native_verdict(&format!("{}\n", receipt::NATIVE_PASS)).is_ok());
}
