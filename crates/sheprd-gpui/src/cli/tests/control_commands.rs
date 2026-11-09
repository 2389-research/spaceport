use super::*;
use control::{
    AppCommand, ConfigCommand, ControlCommand, FontSize, Invocation, NotesCommand, PortsCommand,
    Size, Switch, UiCommand, WaitFor, WindowCommand,
};

fn parse(args: &[&str]) -> Result<Invocation, CliError> {
    LaunchOptions::parse(args.iter().copied()).map(|options| match options.mode {
        LaunchMode::Control(invocation) => invocation,
        mode => panic!("{args:?} parsed as {mode:?}"),
    })
}

fn command(args: &[&str]) -> ControlCommand {
    parse(args).unwrap().command
}

#[test]
fn global_options_apply_anywhere_after_the_command() {
    let invocation = parse(&["window", "--json", "list", "--window", "7"]).unwrap();
    assert_eq!(
        invocation,
        Invocation {
            command: ControlCommand::Window(WindowCommand::List),
            json: true,
            window: Some(7),
        }
    );
    // A bare family runs its listing.
    assert_eq!(
        command(&["tab"]),
        ControlCommand::Tab(control::TabCommand::List)
    );
    assert_eq!(
        parse(&["window", "--window"]).unwrap_err(),
        CliError::MissingValue("--window")
    );
}

#[test]
fn help_wins_over_every_other_argument() {
    for args in [&["ui", "--help"][..], &["notes", "set", "x", "-h"]] {
        assert!(
            matches!(command(args), ControlCommand::Help(Some(_))),
            "{args:?}"
        );
    }
}

#[test]
fn app_window_and_ui_commands() {
    assert_eq!(
        command(&["app", "quit"]),
        ControlCommand::App(AppCommand::Quit)
    );
    assert_eq!(
        command(&["window", "fullscreen", "off"]),
        ControlCommand::Window(WindowCommand::Fullscreen(Switch::Off))
    );
    assert_eq!(
        command(&["window", "size", "1200x800"]),
        ControlCommand::Window(WindowCommand::Size(Some(Size {
            width: 1200,
            height: 800
        })))
    );
    assert_eq!(
        parse(&["window", "size", "12x8"]).unwrap_err(),
        CliError::InvalidValue("WIDTHxHEIGHT")
    );
    assert_eq!(
        command(&["ui", "sidebar"]),
        ControlCommand::Ui(UiCommand::Sidebar(Switch::Toggle))
    );
    assert_eq!(
        command(&["ui", "font-size", "-"]),
        ControlCommand::Ui(UiCommand::FontSize(FontSize::Smaller))
    );
    assert_eq!(
        command(&["ui", "font-size", "14"]),
        ControlCommand::Ui(UiCommand::FontSize(FontSize::Points { points: 14 }))
    );
    assert_eq!(
        parse(&["ui", "font-size", "99"]).unwrap_err(),
        CliError::InvalidValue("POINTS (8-48)")
    );
    assert_eq!(
        parse(&["app", "explode"]).unwrap_err(),
        CliError::UnknownCommand {
            family: "app",
            command: "explode".into()
        }
    );
}

#[test]
fn removed_code_open_command_is_rejected() {
    assert_eq!(
        parse(&["code", "open", "src/main.rs:42"]).unwrap_err(),
        CliError::UnknownOption("code".into())
    );
}

#[test]
fn work_commands() {
    assert_eq!(
        command(&["notes", "set", "waiting", "on", "review"]),
        ControlCommand::Notes(NotesCommand::Set("waiting on review".into()))
    );
    for args in [
        &["scripts"][..],
        &["scripts", "list"],
        &["scripts", "setup"],
        &["scripts", "run"],
    ] {
        assert_eq!(
            parse(args).unwrap_err(),
            CliError::UnknownOption("scripts".into())
        );
    }
    assert_eq!(
        command(&["wait", "tab-closed", "4", "--timeout", "5"]),
        ControlCommand::Wait {
            what: WaitFor::TabClosed(4),
            timeout: 5
        }
    );
}

#[test]
fn excluded_fan_out_command_is_rejected() {
    assert_eq!(
        parse(&["fan-out", "--agent", "claude:2", "fix it"]).unwrap_err(),
        CliError::UnknownOption("fan-out".into())
    );
}

#[test]
fn host_and_settings_commands() {
    assert_eq!(
        command(&["ports", "forward", "3000"]),
        ControlCommand::Ports(PortsCommand::Forward(3000))
    );
    assert_eq!(
        parse(&["ports", "open", "0"]).unwrap_err(),
        CliError::InvalidValue("PORT")
    );
    assert_eq!(
        command(&["config", "set", "terminal.size", "14"]),
        ControlCommand::Config(ConfigCommand::Set {
            key: "terminal.size".into(),
            value: "14".into()
        })
    );
    assert_eq!(
        parse(&["config", "get", "a..b"]).unwrap_err(),
        CliError::InvalidValue("KEY")
    );
    assert_eq!(
        command(&["caffeine", "on", "--for", "30m"]),
        ControlCommand::Caffeine(control::CaffeineCommand::Set {
            switch: Switch::On,
            seconds: Some(1800)
        })
    );
    assert_eq!(
        command(&["caffeine"]),
        ControlCommand::Caffeine(control::CaffeineCommand::Status)
    );
    assert_eq!(
        parse(&["caffeine", "off", "--for", "1h"]).unwrap_err(),
        CliError::InvalidValue("--for with on or toggle")
    );
    assert_eq!(
        command(&[
            "events",
            "--kind",
            "tab-opened",
            "--kind",
            "note",
            "--wait",
            "5"
        ]),
        ControlCommand::Events {
            after: None,
            follow: false,
            wait: 5,
            kinds: vec![control::EventKind::TabOpened, control::EventKind::Note],
        }
    );
    assert_eq!(
        parse(&["events", "--kind", "nope"]).unwrap_err(),
        CliError::InvalidValue("--kind")
    );
    assert_eq!(
        parse(&["logs", "extra"]).unwrap_err(),
        CliError::Unexpected {
            command: "logs",
            argument: "extra".into()
        }
    );
}

#[test]
fn an_events_cursor_names_its_app_run() {
    assert_eq!(
        command(&["events", "--after", "77:12"]),
        ControlCommand::Events {
            after: Some(control::Cursor {
                instance: Some(77),
                seq: 12
            }),
            follow: false,
            wait: 0,
            kinds: Vec::new(),
        }
    );
    let ControlCommand::Events { after, .. } = command(&["events", "--after", "12"]) else {
        panic!("not events");
    };
    assert_eq!(
        after.map(|cursor| cursor.to_string()).as_deref(),
        Some("12")
    );
    assert_eq!(
        parse(&["events", "--after", "x:1"]).unwrap_err(),
        CliError::InvalidValue("--after")
    );
}

#[test]
fn mute_commands() {
    assert_eq!(
        command(&["host", "mute", "studio"]),
        ControlCommand::Host(control::HostCommand::Mute {
            host: Some("studio".into()),
            muted: true
        })
    );
    assert_eq!(
        command(&["host", "unmute"]),
        ControlCommand::Host(control::HostCommand::Mute {
            host: None,
            muted: false
        })
    );
    for (words, expected) in [
        (&["worktree"][..], control::WorktreeCommand::Status),
        (&["worktree", "status"], control::WorktreeCommand::Status),
        (&["worktree", "mute"], control::WorktreeCommand::Mute),
        (&["worktree", "unmute"], control::WorktreeCommand::Unmute),
    ] {
        assert_eq!(command(words), ControlCommand::Worktree(expected));
    }
    assert!(parse(&["worktree", "silence"]).is_err());
}

#[test]
fn host_tag_takes_a_host_and_any_number_of_tags() {
    assert_eq!(
        command(&["host", "tag", "devbox", "work", "client acme"]),
        ControlCommand::Host(control::HostCommand::Tag {
            id: "devbox".into(),
            tags: vec!["work".into(), "client acme".into()],
        })
    );
    assert_eq!(
        command(&["host", "tag", "devbox"]),
        ControlCommand::Host(control::HostCommand::Tag {
            id: "devbox".into(),
            tags: Vec::new(),
        })
    );
    assert!(parse(&["host", "tag"]).is_err());
}
