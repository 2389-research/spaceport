// ABOUTME: Keep the fixture registry a run can resolve to exactly one fixture.
// ABOUTME: Catch a registration that two variables select or that nothing does.
use super::{FIXTURES, OWNED_ELSEWHERE};
use std::collections::HashSet;

#[test]
fn every_fixture_variable_is_registered_once() {
    let mut names = HashSet::new();
    for fixture in FIXTURES.iter().chain(OWNED_ELSEWHERE) {
        if let Some(variable) = fixture.variable {
            assert!(names.insert(variable), "{variable} selects two fixtures");
        }
    }
}

#[test]
fn exactly_one_fixture_runs_without_a_variable() {
    let defaults = FIXTURES
        .iter()
        .chain(OWNED_ELSEWHERE)
        .filter(|fixture| fixture.default_fixture())
        .count();
    assert_eq!(defaults, 1, "a run without a variable needs one default");
}
