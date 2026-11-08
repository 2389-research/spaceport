// ABOUTME: Checks local CLI source-search ranking without the removed picker UI.
// ABOUTME: Keeps file and symbol results bounded and stable across the view cut.
use super::*;
use crate::code_index::{Change, Index, Kind, Symbol};
use std::path::Path;

fn symbol(name: &str, kind: Kind, file: usize, line: u32) -> Symbol {
    Symbol {
        name: name.into(),
        kind,
        file,
        line,
    }
}

fn fixture() -> Index {
    Index::of(
        Path::new("/repo"),
        &["src/code_search.rs", "src/main.rs", "src/window/render.rs"],
        vec![
            symbol("render_code_search", Kind::Method, 0, 300),
            symbol("CodeSearch", Kind::Type, 0, 60),
            symbol("main", Kind::Function, 1, 1),
            symbol("render", Kind::Method, 2, 12),
        ],
    )
}

fn labels(index: &Index, mode: Mode, query: &str) -> Vec<String> {
    rank(index, mode, &Query::parse(query, mode))
        .iter()
        .map(|hit| label(index, mode, hit.item).to_owned())
        .collect()
}

#[test]
fn symbols_rank_by_match_then_by_shorter_name() {
    let index = fixture();
    assert_eq!(
        labels(&index, Mode::Symbols, "render"),
        ["render", "render_code_search"]
    );
    assert_eq!(
        labels(&index, Mode::Symbols, "cdsrch"),
        ["CodeSearch", "render_code_search"]
    );
    assert_eq!(labels(&index, Mode::Symbols, "  ").len(), 4);
    assert!(labels(&index, Mode::Symbols, "zzz").is_empty());
    let hits = rank(&index, Mode::Symbols, &Query::parse("main", Mode::Symbols));
    assert_eq!(hits[0].highlights, [0..4]);
}

#[test]
fn files_match_paths_and_take_a_line() {
    let index = fixture();
    assert_eq!(
        labels(&index, Mode::Files, "render"),
        ["src/window/render.rs"]
    );
    let query = Query::parse("main.rs:42", Mode::Files);
    assert_eq!((query.text.as_str(), query.line), ("main.rs", Some(42)));
    assert_eq!(labels(&index, Mode::Files, "main.rs:42"), ["src/main.rs"]);
    assert_eq!(Query::parse("a:1", Mode::Symbols).line, None);
}

#[test]
fn without_a_query_the_changed_files_come_first() {
    let change = |file: usize| Change {
        file,
        counts: Some((1, 1)),
        new: false,
        modified: None,
    };
    let index = fixture().with_changes(vec![change(2), change(0)]);
    assert_eq!(
        labels(&index, Mode::Files, ""),
        ["src/window/render.rs", "src/code_search.rs", "src/main.rs"]
    );
    assert_eq!(
        labels(&index, Mode::Symbols, ""),
        ["render", "render_code_search", "CodeSearch", "main"]
    );
    assert_eq!(labels(&index, Mode::Files, "main"), ["src/main.rs"]);
}

#[test]
fn hits_are_bounded() {
    let names: Vec<String> = (0..MAX_HITS + 50).map(|i| format!("item{i}")).collect();
    let files: Vec<&str> = names.iter().map(String::as_str).collect();
    let index = Index::of(Path::new("/repo"), &files, Vec::new());
    for query in ["", "item"] {
        assert_eq!(labels(&index, Mode::Files, query).len(), MAX_HITS);
    }
}
