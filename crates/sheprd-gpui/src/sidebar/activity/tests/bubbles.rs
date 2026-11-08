//! Activity counts and visit stamps keep changes visible until seen.
use super::*;
use crate::sidebar::activity::counts::Counts;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

#[test]
fn a_count_is_unseen_until_its_section_opens_then_only_for_newer_items() {
    let mut activity = super::super::Activity::default();
    let earlier = Utc::now() - Duration::hours(1);
    let later = Utc::now() + Duration::hours(1);
    let panel = ActivityPanel::PullRequests;
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
    assert!(activity.unseen(ActivityPanel::Issues, Some(earlier)));
}

#[test]
fn stored_stamps_merge_without_moving_a_later_visit_back() {
    let mut activity = super::super::Activity::default();
    activity.pick(ActivityPanel::Issues);
    activity.pick(ActivityPanel::Workspaces);
    let visited = activity.last_opened()[&ActivityPanel::Issues];
    let stale = visited - Duration::days(1);
    let stored: HashMap<_, _> = [
        (ActivityPanel::Issues, stale),
        (ActivityPanel::Tasks, stale),
    ]
    .into();
    // This run's visits are newer than the stored ones, so the merge differs
    // from the file and must be saved.
    assert!(activity.restore(None, &stored));
    assert_eq!(activity.last_opened()[&ActivityPanel::Issues], visited);
    assert_eq!(activity.last_opened()[&ActivityPanel::Tasks], stale);
    assert!(!activity.unseen(ActivityPanel::Tasks, Some(stale)));
    assert!(activity.unseen(ActivityPanel::Tasks, Some(stale + Duration::seconds(1))));
}

#[test]
fn counts_keep_their_newest_item_time() {
    let at = |hour: i64| DateTime::<Utc>::from_timestamp(hour * 3600, 0).unwrap();
    let mut updated = item("1", "Updated");
    updated.created_at = Some(at(1));
    updated.updated_at = Some(at(5));
    let mut created = item("2", "Only created");
    created.created_at = Some(at(3));
    let mut closed = item("3", "Closed later");
    closed.state = "closed".into();
    closed.updated_at = Some(at(9));
    let mut pull = item("pr/4", "Pull request");
    pull.pull_request = Some(pull_request(4));
    let mut counts = Counts::default();
    counts.add(&mut HashSet::new(), &[updated, created, closed, pull]);
    // A closed item is not counted, so its change does not light the bubble.
    assert_eq!(counts.latest(ActivityPanel::Issues), Some(at(5)));
    // An item without times counts but cannot be told new.
    assert_eq!(counts.of(ActivityPanel::PullRequests), Some(1));
    assert_eq!(counts.latest(ActivityPanel::PullRequests), None);
    assert_eq!(counts.latest(ActivityPanel::Usage), None);
}

#[test]
fn a_newer_copy_in_a_later_view_still_lights_the_count() {
    let at = |hour: i64| DateTime::<Utc>::from_timestamp(hour * 3600, 0).unwrap();
    let mut stale = item("1", "Synced earlier");
    stale.updated_at = Some(at(1));
    let mut fresh = stale.clone();
    fresh.updated_at = Some(at(4));
    let mut seen = HashSet::new();
    let mut counts = Counts::default();
    counts.add(&mut seen, &[stale]);
    counts.add(&mut seen, &[fresh]);
    assert_eq!(counts.of(ActivityPanel::Issues), Some(1));
    assert_eq!(counts.latest(ActivityPanel::Issues), Some(at(4)));
}
