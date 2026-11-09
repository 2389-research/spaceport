//! Activity counts and visit stamps keep changes visible until seen.
use super::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;

#[test]
fn a_count_is_unseen_until_its_section_opens_then_only_for_newer_items() {
    let mut activity = super::super::Activity::default();
    let earlier = Utc::now() - Duration::hours(1);
    let later = Utc::now() + Duration::hours(1);
    let panel = ActivityPanel::Overview;
    // Never opened: whatever it counts is new.
    assert!(activity.unseen(panel, None));
    assert!(activity.unseen(panel, Some(earlier)));
    // Shown: the user is looking at it.
    assert!(activity.pick(panel));
    assert!(!activity.unseen(panel, Some(later)));
    // Left: what changed while it was open was seen, what changes after is not.
    assert!(activity.pick(ActivityPanel::Workspaces));
    assert!(!activity.unseen(panel, Some(earlier)));
    assert!(!activity.unseen(panel, None));
    assert!(activity.unseen(panel, Some(later)));
    // Other sections keep their own stamps.
    assert!(activity.unseen(ActivityPanel::Usage, Some(earlier)));
}

#[test]
fn stored_stamps_merge_without_moving_a_later_visit_back() {
    let mut activity = super::super::Activity::default();
    activity.pick(ActivityPanel::Overview);
    activity.pick(ActivityPanel::Workspaces);
    let visited = activity.last_opened()[&ActivityPanel::Overview];
    let stale = visited - Duration::days(1);
    let stored: HashMap<_, _> = [
        (ActivityPanel::Overview, stale),
        (ActivityPanel::Usage, stale),
    ]
    .into();
    // This run's visits are newer than the stored ones, so the merge differs
    // from the file and must be saved.
    assert!(activity.restore(None, &stored));
    assert_eq!(activity.last_opened()[&ActivityPanel::Overview], visited);
    assert_eq!(activity.last_opened()[&ActivityPanel::Usage], stale);
    assert!(!activity.unseen(ActivityPanel::Usage, Some(stale)));
    assert!(activity.unseen(ActivityPanel::Usage, Some(stale + Duration::seconds(1))));
}
