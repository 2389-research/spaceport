use super::*;
use crate::GroupIds;

#[test]
fn a_split_opens_from_its_source_and_a_close_folds_into_its_neighbour() {
    let mut ids = GroupIds::default();
    let (a, b) = (ids.next(), ids.next());
    let mut motion = GroupMotion::default();
    motion.enable();
    let start = Instant::now();
    // Settled, a and b each take half the row.
    let settled = |_| 0.5;
    motion.open(b, a, start);
    assert!(motion.animating(start));
    assert_eq!(motion.opened(b, start), Some(0.));
    // At the start the new group has no room and its source has it all.
    assert_eq!(motion.share(b, 0.5, settled, start), 0.);
    assert_eq!(motion.share(a, 0.5, settled, start), 1.);
    // Mid-way the room is shared, always summing to the whole row.
    let mid = start + ENTER / 2;
    let (sa, sb) = (
        motion.share(a, 0.5, settled, mid),
        motion.share(b, 0.5, settled, mid),
    );
    assert!(
        (sa + sb - 1.).abs() < 1e-5 && sb > 0. && sb < 0.5,
        "{sa} {sb}"
    );
    assert_eq!(motion.share(b, 0.5, settled, start + ENTER), 0.5);
    assert!(!motion.animating(start + ENTER));

    // Closing b gives a its room back gradually.
    motion.fold(b, 1, 0.5, a, start);
    assert_eq!(motion.folding(start)[0].share, 0.5);
    assert_eq!(motion.share(a, 1., |_| 1., start), 0.5);
    assert_eq!(motion.share(a, 1., |_| 1., start + LEAVE), 1.);
    assert!(motion.folding(start + LEAVE).is_empty());
    assert!(!motion.animating(start + LEAVE));

    // Closed while still opening, it folds from the room it had.
    let later = start + LEAVE;
    let c = ids.next();
    motion.open(c, a, later);
    motion.fold(c, 1, 0.5, a, later + ENTER / 2);
    let from = motion.folding(later + ENTER / 2)[0].share;
    assert!(from > 0. && from < 0.5, "{from}");
    assert_eq!(motion.opened(c, later + ENTER / 2), None);
}
