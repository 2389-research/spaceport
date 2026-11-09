// ABOUTME: Checks that animated native tabs retain a final reveal attempt.
// ABOUTME: Bounds retries after growth so an unreadable layout cannot spin forever.
use super::*;
use crate::browser::GroupIds;

#[test]
fn growth_does_not_spend_the_final_layout_reveal_budget() {
    let group = GroupIds::default().next();
    let pick = Pick::Page(1.into());
    let mut scroll = TabScroll::default();
    for _ in 0..=MAX_TRIES {
        assert!(scroll.reveal(group, &pick, 0, 100., true));
    }
    assert!(scroll.reveal(group, &pick, 0, 100., false));
    for _ in 1..MAX_TRIES {
        scroll.reveal(group, &pick, 0, 100., false);
    }
    assert!(!scroll.reveal(group, &pick, 0, 100., false));
}
