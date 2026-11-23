use super::*;

#[test]
fn a_drop_lands_before_the_device_holding_the_gap_or_after_the_last() {
    let ids: Vec<String> = ["a", "b", "c"].map(String::from).into();
    // The gaps around the carried device move nothing.
    assert_eq!(landing(&ids, 1, 1), None);
    assert_eq!(landing(&ids, 1, 2), None);
    assert_eq!(landing(&ids, 1, 0), Some(Beside::Before("a".into())));
    assert_eq!(landing(&ids, 0, 2), Some(Beside::Before("c".into())));
    assert_eq!(landing(&ids, 0, 3), Some(Beside::After("c".into())));
}
