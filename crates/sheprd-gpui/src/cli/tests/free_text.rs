// ABOUTME: Checks that free text survives CLI option parsing.
// ABOUTME: Covers note text, config values, and explicit option boundaries.
use super::*;
use control::{ConfigCommand, ControlCommand, Invocation, ThemeCommand, WorkOp};

fn parse(args: &[&str]) -> Invocation {
    match LaunchOptions::parse(args.iter().copied()).unwrap().mode {
        LaunchMode::Control(invocation) => invocation,
        mode => panic!("{args:?} parsed as {mode:?}"),
    }
}

#[test]
fn note_text_preserves_flag_words_after_text_starts() {
    for (words, text) in [
        (vec!["rerun", "with", "-h"], "rerun with -h"),
        (vec!["fix", "--force", "push"], "fix --force push"),
        (
            vec!["use", "--json", "--window", "7", "--help"],
            "use --json --window 7 --help",
        ),
        (vec!["keep", "--", "--json"], "keep -- --json"),
    ] {
        let mut args = vec!["notes", "set"];
        args.extend(words);
        assert_eq!(
            parse(&args),
            Invocation {
                command: ControlCommand::Notes(WorkOp::SetNote { text: text.into() }),
                json: false,
                window: None,
            }
        );
    }
}

#[test]
fn shared_options_still_apply_before_free_text() {
    let invocation = parse(&["notes", "--window", "7", "set", "--json", "fix", "--json"]);
    assert_eq!(
        invocation,
        Invocation {
            command: ControlCommand::Notes(WorkOp::SetNote {
                text: "fix --json".into()
            }),
            json: true,
            window: Some(7),
        }
    );
}

#[test]
fn config_values_allow_negative_numbers_and_literal_options() {
    for (words, value) in [
        (vec!["-1"], "-1"),
        (vec!["rerun", "--json", "--help"], "rerun --json --help"),
        (vec!["--", "--json"], "--json"),
    ] {
        let mut args = vec!["config", "set", "terminal.size"];
        args.extend(words);
        assert_eq!(
            parse(&args).command,
            ControlCommand::Config(ConfigCommand::Set {
                key: "terminal.size".into(),
                value: value.into(),
            })
        );
    }
}

#[test]
fn theme_and_font_names_preserve_flag_words() {
    for family in ["theme", "font"] {
        let invocation = parse(&[family, "set", "Some", "--json", "Font"]);
        let name = ThemeCommand::Set(Some("Some --json Font".into()));
        assert_eq!(
            invocation.command,
            if family == "theme" {
                ControlCommand::Theme(name)
            } else {
                ControlCommand::Font(name)
            }
        );
        assert!(!invocation.json);
    }
}

#[test]
fn double_dash_allows_literal_help_and_json_text() {
    assert_eq!(
        parse(&["notes", "set", "--", "--help", "--json"]).command,
        ControlCommand::Notes(WorkOp::SetNote {
            text: "--help --json".into()
        })
    );
    assert_eq!(
        parse(&["notes", "--", "set", "--json"]).command,
        ControlCommand::Notes(WorkOp::SetNote {
            text: "--json".into()
        })
    );
    assert_eq!(parse(&["window", "focus", "--", "7"]).window, Some(7));
}

#[test]
fn json_error_detection_respects_the_same_text_boundary() {
    for (words, expected) in [
        (vec!["notes", "set", "rerun", "--json"], false),
        (vec!["notes", "set", "--json", "rerun"], true),
        (
            vec!["config", "set", "terminal.size", "-1", "--json"],
            false,
        ),
        (vec!["notes", "set", "--", "--json"], false),
        (vec!["tab", "focus", "x", "--json"], true),
    ] {
        assert_eq!(
            control::wants_json(&words.iter().map(OsString::from).collect::<Vec<_>>()),
            expected,
            "{words:?}"
        );
    }
}

#[test]
fn double_dash_still_ends_options_after_a_missing_option_value() {
    assert_eq!(
        LaunchOptions::parse(["notes", "set", "--window", "--", "--help"]).unwrap_err(),
        CliError::MissingValue("--window")
    );
}
