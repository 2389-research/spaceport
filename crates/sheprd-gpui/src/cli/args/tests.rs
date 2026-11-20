// ABOUTME: Tests control argument cursor behavior and option validation.
// ABOUTME: Checks literal tails and unconsumed argument errors.
#![allow(clippy::unwrap_used)]
use super::*;

fn args(words: &[&str]) -> Args {
    Args::new(words.iter().map(OsString::from), "test")
}

#[test]
fn options_are_found_anywhere_and_values_cannot_be_flags() {
    let mut parsed = args(&["a", "--wait", "5", "--json", "b"]);
    assert!(parsed.flag("--json"));
    assert_eq!(parsed.number("--wait", 1..=10u64).unwrap(), Some(5));
    assert_eq!(parsed.required_text("x").unwrap(), "a");
    assert_eq!(parsed.optional_text().unwrap().as_deref(), Some("b"));
    parsed.finish().unwrap();
    assert_eq!(
        args(&["--workspace", "--json"])
            .option("--workspace")
            .unwrap_err(),
        CliError::MissingValue("--workspace")
    );
    assert_eq!(
        args(&["--wait"]).option("--wait").unwrap_err(),
        CliError::MissingValue("--wait")
    );
    assert_eq!(
        args(&["--wait", "1", "--wait", "2"])
            .option("--wait")
            .unwrap_err(),
        CliError::DuplicateOption("--wait")
    );
    assert_eq!(
        args(&["--wait", "99"])
            .number("--wait", 1..=10u64)
            .unwrap_err(),
        CliError::InvalidValue("--wait")
    );
}

#[test]
fn leftovers_and_missing_arguments_are_errors() {
    assert_eq!(
        args(&["--new"]).finish().unwrap_err(),
        CliError::Unexpected {
            command: "test",
            argument: "--new".into()
        }
    );
    assert_eq!(
        args(&[]).required("FILE").unwrap_err(),
        CliError::MissingArgument {
            command: "test",
            what: "FILE"
        }
    );
    let mut words = args(&["fix", "the", "bug", "--agents", "2"]);
    assert_eq!(words.number("--agents", 1..=8u8).unwrap(), Some(2));
    assert_eq!(words.words().unwrap().as_deref(), Some("fix the bug"));
    // A lone dash is a value, as for stdin.
    assert_eq!(args(&["-"]).required_text("x").unwrap(), "-");
}

#[test]
fn double_dash_words_are_not_parsed_as_options() {
    let mut parsed = args(&["id", "--", "git", "log", "--json", "--"]);
    assert!(!parsed.flag("--json"));
    assert!(!parsed.wants_help());
    assert_eq!(parsed.required_text("ID").unwrap(), "id");
    assert_eq!(
        parsed.finish().unwrap_err(),
        CliError::Unexpected {
            command: "test",
            argument: "git".into(),
        }
    );
    // Unread literal arguments are refused, while the separator is consumed.
    assert!(args(&["--", "x"]).finish().is_err());
    args(&["--"]).finish().unwrap();
}
