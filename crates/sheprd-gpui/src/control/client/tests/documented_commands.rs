//! The CLI document and the agent skill are contracts with scripts and
//! agents: a script branches on the exit statuses they print and an agent
//! copies their command lines, so both must name the families this binary
//! parses and the protocol version it speaks.
use crate::cli::control::{FAMILIES, is_family, parse};
use std::ffi::OsString;

const CLI_DOC: &str = include_str!("../../../../../../docs/cli.md");
const SKILL: &str = include_str!("../../../../../../skills/spaceport/SKILL.md");

/// Whether `doc` documents `name` as a command, which it writes in backticks.
fn documents(doc: &str, name: &str) -> bool {
    doc.contains(&format!("`{name} ")) || doc.contains(&format!("`{name}`"))
}

/// The `spaceport ...` command lines the skill writes in backticks.
fn skill_command_lines() -> Vec<&'static str> {
    SKILL
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| span.starts_with("spaceport "))
        .collect()
}

/// A placeholder such as `ID` or `KEY` stands for a word the reader picks;
/// `1` parses wherever a real one would.
fn sample(word: &str) -> &str {
    if word.chars().all(|c| c.is_ascii_uppercase()) {
        "1"
    } else {
        word
    }
}

#[test]
fn the_cli_doc_names_every_family() {
    for family in FAMILIES {
        assert!(
            documents(CLI_DOC, family.name),
            "docs/cli.md documents no `{}` command",
            family.name
        );
    }
}

#[test]
fn the_cli_doc_states_the_protocol_version_the_app_speaks() {
    let claim = CLI_DOC
        .split_once("protocol version, currently ")
        .map(|(_, claim)| claim)
        .unwrap_or_else(|| panic!("docs/cli.md does not say `protocol version, currently N`"));
    let digits: String = claim.chars().take_while(char::is_ascii_digit).collect();
    let documented: u32 = digits
        .parse()
        .unwrap_or_else(|_| panic!("docs/cli.md says `protocol version, currently {claim}`"));
    assert_eq!(documented, crate::control::protocol::PROTOCOL_VERSION);
}

#[test]
fn every_skill_command_line_parses() {
    let lines = skill_command_lines();
    assert!(!lines.is_empty(), "the skill shows no command line");
    for line in lines {
        let mut words = line.split_whitespace();
        assert_eq!(words.next(), Some("spaceport"), "`{line}`");
        let word = words
            .next()
            .unwrap_or_else(|| panic!("`{line}` names no command"));
        // These two name the program rather than one of its families.
        if matches!(word, "--help" | "COMMAND") {
            continue;
        }
        let family = OsString::from(word);
        assert!(is_family(&family), "`{line}` starts with no family");
        let rest = words.map(|word| OsString::from(sample(word)));
        if let Err(error) = parse(family, rest) {
            panic!("`{line}` does not parse: {error}");
        }
    }
}
