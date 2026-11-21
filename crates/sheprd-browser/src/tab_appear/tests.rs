use super::*;
use crate::GroupIds;

fn herdr(id: &str) -> Pick {
    Pick::Herdr(id.into())
}

fn listed(ids: &[&str]) -> Vec<Listed> {
    ids.iter()
        .map(|id| Listed {
            pick: herdr(id),
            label: SharedString::from(id.to_string()),
            width: 120.,
        })
        .collect()
}

#[test]
fn tabs_grow_in_and_shrink_out_of_their_own_strip() {
    let mut appear = TabAppear::default();
    let mut ids = GroupIds::default();
    let (left, right) = (ids.next(), ids.next());
    let start = Instant::now();
    // A strip's tabs when first seen simply appear.
    appear.observe(left, listed(&["t1", "t2"]), start);
    appear.observe(right, listed(&["t1", "t2"]), start);
    assert!(!appear.animating());

    // A new one grows in, eased out, and ends whole.
    appear.observe(left, listed(&["t1", "t2", "t3"]), start);
    assert!(appear.animating());
    assert_eq!(appear.growth(left, &herdr("t3"), start), Some(0.));
    assert!(appear.growth(right, &herdr("t3"), start).is_none());
    assert_eq!(appear.growth(left, &herdr("t3"), start + ENTER), None);

    // A closed one shrinks out where it stood, in that strip only.
    appear.observe(right, listed(&["t2"]), start);
    let leaving = appear.leaving(right);
    assert_eq!(leaving.len(), 1);
    assert_eq!(
        (leaving[0].pick.clone(), leaving[0].index),
        (herdr("t1"), 0)
    );
    assert_eq!(leaving[0].left(start), 1.);
    assert_eq!(leaving[0].left(start + LEAVE), 0.);
    assert!(appear.leaving(left).is_empty());
    appear.observe(right, listed(&["t2"]), start + LEAVE);
    assert!(appear.leaving(right).is_empty());

    // A tab that comes back while leaving grows in instead.
    appear.observe(left, listed(&["t2", "t3"]), start + ENTER);
    appear.observe(left, listed(&["t1", "t2", "t3"]), start + ENTER);
    assert!(appear.leaving(left).is_empty());
    assert!(appear.growth(left, &herdr("t1"), start + ENTER).is_some());

    // A gone group's strip is forgotten: its next sight grows nothing.
    appear.retain(|group| group == left);
    appear.observe(right, listed(&["t9"]), start + ENTER);
    assert!(appear.leaving(right).is_empty());
    assert!(appear.growth(right, &herdr("t9"), start + ENTER).is_none());
}
