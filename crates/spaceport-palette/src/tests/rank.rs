use crate::{Action, AgentFilter, Entry, Filter, Project, rank_rows, search};

fn project(label: &str, boost: u32) -> Entry {
    let mut entry = Entry::new(
        label.into(),
        String::new(),
        "Project",
        Action::Project(Project {
            path: format!("/code/{label}").into(),
            label: label.into(),
        }),
        None,
    );
    entry.boost = boost;
    entry
}

fn rows(filter: Filter, entries: &[Entry], query: &str) -> Vec<String> {
    let hits = rank_rows(
        filter,
        AgentFilter::All,
        entries,
        (0..entries.len()).collect(),
        &search::Query::parse(query),
        &mut search::matcher(),
    );
    hits.iter()
        .map(|hit| entries[hit.index].label.to_string())
        .collect()
}

#[test]
fn a_frecent_row_rises_among_matches_but_never_makes_a_match() {
    let entries = [
        project("build-a", 0),
        project("build-b", 32),
        project("other", 64),
    ];
    assert_eq!(rows(Filter::All, &entries, "build"), ["build-b", "build-a"]);
    let unboosted = [project("build-a", 0), project("build-b", 0)];
    assert_eq!(
        rows(Filter::All, &unboosted, "build"),
        ["build-a", "build-b"]
    );
}

#[test]
fn only_the_flat_project_list_sorts_by_frecency_without_a_query() {
    let entries = [
        project("a", 0),
        project("b", 40),
        project("c", 0),
        project("d", 40),
    ];
    assert_eq!(rows(Filter::Projects, &entries, ""), ["b", "d", "a", "c"]);
    // Elsewhere an empty query keeps entry order, so Go To keeps its tree.
    assert_eq!(rows(Filter::All, &entries, ""), ["a", "b", "c", "d"]);
}
