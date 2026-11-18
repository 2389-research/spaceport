// ABOUTME: Verifies that text probes never acquire unrelated image content.
// ABOUTME: Keeps empty text probes distinct from normal attachment pastes.
use super::*;

#[test]
fn a_text_probe_stops_after_missing_text() -> Result<()> {
    let mut formats = Vec::new();
    let item = read_with(Mode::Text, |format, _| {
        formats.push(format);
        Ok(format.map(|_| vec![1, 2, 3]))
    })?;
    assert!(item.is_none(), "a text probe returned image content");
    assert_eq!(formats, [None]);
    Ok(())
}
