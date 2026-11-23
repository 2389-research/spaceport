use super::*;

fn tagged(records: &[(&str, &[&str])]) -> Tags {
    Tags::tagged(records)
}

/// A filter outlives its segment only while another tag keeps the switch
/// shown and the tag itself is still in use somewhere.
#[test]
fn a_filter_with_no_segment_left_to_clear_it_is_stale() {
    let one = tagged(&[("a", &["work"])]);
    assert!(filter_stale(&one, "work"));
    let two = tagged(&[("a", &["work"]), ("b", &["lab"])]);
    assert!(!filter_stale(&two, "lab"));
    // Whatever its case, and gone once no device carries it.
    assert!(!filter_stale(&two, "LAB"));
    assert!(filter_stale(&two, "gone"));
    assert!(filter_stale(&Tags::default(), "work"));
}

#[test]
fn devices_section_together_whatever_their_case() {
    assert!(same_section(Some("work"), Some("Work")));
    assert!(same_section(None, None));
    assert!(!same_section(Some("work"), Some("lab")));
    assert!(!same_section(Some("work"), None));
    assert!(!same_section(None, Some("work")));
}
