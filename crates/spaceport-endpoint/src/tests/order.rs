use super::*;
use spaceport_reorder::Beside;

#[test]
fn unarranged_devices_rank_after_arranged_ones() {
    let order = ["ssh:b".to_owned(), "ssh:a".to_owned()];
    assert_eq!(rank(&order, "ssh:b"), 0);
    assert_eq!(rank(&order, "ssh:a"), 1);
    assert_eq!(rank(&order, "ssh:c"), 2);
    assert_eq!(rank(&order, "wsl:d"), 2);
}

#[test]
fn a_move_lands_beside_its_anchor() {
    let ids = ["a", "b", "c", "d"];
    assert_eq!(
        moved(&ids, "d", Beside::Before("b")).unwrap(),
        ["a", "d", "b", "c"]
    );
    assert_eq!(
        moved(&ids, "a", Beside::After("c")).unwrap(),
        ["b", "c", "a", "d"]
    );
    assert_eq!(
        moved(&ids, "a", Beside::After("d")).unwrap(),
        ["b", "c", "d", "a"]
    );
    assert_eq!(moved(&ids, "e", Beside::Before("a")), None);
    assert_eq!(moved(&ids, "a", Beside::Before("e")), None);
    assert_eq!(moved(&ids, "a", Beside::Before("a")), None);
}
