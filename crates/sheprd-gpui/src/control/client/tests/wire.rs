//! What each command of the families that are one protocol operation puts
//! on the wire: the command line goes through the real parser and the real
//! client function, and the request the client would send is recorded
//! instead of sent. `wire.txt` holds, for every row, the request as JSON
//! without its caller, the window it targets, and whether `--json` was
//! asked for, or the parse error. Update it only with a deliberate change
//! to what a command sends.
use super::*;
use crate::cli::{LaunchMode, LaunchOptions};
use std::cell::RefCell;

thread_local! {
    static SENT: RefCell<Vec<Request>> = const { RefCell::new(Vec::new()) };
}

/// Stands in for the app: keeps the request and answers `Done`.
fn record(request: &Request) -> crate::Result<Response> {
    SENT.with_borrow_mut(|sent| sent.push(request.clone()));
    Ok(Response::Done {
        message: String::new(),
    })
}

/// The longest note text the parser takes, in bytes.
const NOTE_LIMIT: usize = 16 * 1024;

/// Command lines, each family's verbs, flags, and refusals.
const ROWS: &[&[&str]] = &[
    // app
    &["app"],
    &["app", "focus"],
    &["app", "hide"],
    &["app", "quit"],
    &["app", "quit", "--json"],
    &["app", "focus", "--window", "2"],
    &["app", "explode"],
    &["app", "focus", "extra"],
    // window
    &["window"],
    &["window", "list"],
    &["window", "list", "--json"],
    &["window", "--json", "list", "--window", "7"],
    &["window", "new"],
    &["window", "focus", "7"],
    &["window", "focus", "7", "--window", "3"],
    &["window", "--window", "3", "focus", "7"],
    &["window", "focus", "7", "--json"],
    &["window", "focus"],
    &["window", "focus", "x"],
    &["window", "focus", "-1"],
    &["window", "focus", "7", "8"],
    &["window", "fullscreen"],
    &["window", "fullscreen", "on"],
    &["window", "fullscreen", "off"],
    &["window", "fullscreen", "toggle"],
    &["window", "fullscreen", "show"],
    &["window", "fullscreen", "hide"],
    &["window", "fullscreen", "maybe"],
    &["window", "size"],
    &["window", "size", "1200x800"],
    &["window", "size", "1200X800"],
    &["window", "size", "200x150"],
    &["window", "size", "16384x16384"],
    &["window", "size", "199x150"],
    &["window", "size", "200x149"],
    &["window", "size", "16385x200"],
    &["window", "size", "12x8"],
    &["window", "size", "wide"],
    &["window", "detach"],
    &["window", "reconnect"],
    &["window", "detach", "--window", "4"],
    &["window", "explode"],
    &["window", "list", "extra"],
    &["window", "--window"],
    &["window", "--window", "x"],
    &["window", "list", "--window", "1", "--window", "2"],
    // ui
    &["ui"],
    &["ui", "sidebar"],
    &["ui", "sidebar", "on"],
    &["ui", "sidebar", "off"],
    &["ui", "sidebar", "toggle"],
    &["ui", "sidebar", "show"],
    &["ui", "sidebar", "hide"],
    &["ui", "sidebar", "maybe"],
    &["ui", "statusbar"],
    &["ui", "statusbar", "on"],
    &["ui", "status-bar", "off"],
    &["ui", "statusbar", "maybe"],
    &["ui", "font-size"],
    &["ui", "font-size", "+"],
    &["ui", "font-size", "larger"],
    &["ui", "font-size", "-"],
    &["ui", "font-size", "smaller"],
    &["ui", "font-size", "reset"],
    &["ui", "font-size", "14"],
    &["ui", "font-size", "8"],
    &["ui", "font-size", "48"],
    &["ui", "font-size", "7"],
    &["ui", "font-size", "49"],
    &["ui", "font-size", "big"],
    &["ui", "panel"],
    &["ui", "panel", "workspaces"],
    &["ui", "panel", "overview"],
    &["ui", "panel", "automations"],
    &["ui", "panel", "usage"],
    &["ui", "panel", "usage", "--json", "--window", "5"],
    &["ui", "panel", "dashboard"],
    &["ui", "panel", "security"],
    &["ui", "explode"],
    &["ui", "sidebar", "on", "extra"],
    // tab
    &["tab"],
    &["tab", "list"],
    &["tab", "focus", "4"],
    &["tab", "close", "4"],
    &["tab", "focus", "4", "--window", "2", "--json"],
    &["tab", "focus"],
    &["tab", "close"],
    &["tab", "focus", "x"],
    &["tab", "close", "-4"],
    &["tab", "open"],
    &["tab", "devices"],
    &["tab", "list", "extra"],
    // host
    &["host"],
    &["host", "list"],
    &["host", "list", "--json", "--window", "4"],
    &["host", "select", "devbox"],
    &["host", "select"],
    &["host", "select", "a", "b"],
    &["host", "mute"],
    &["host", "mute", "studio"],
    &["host", "unmute"],
    &["host", "unmute", "studio"],
    &["host", "mute", "a", "b"],
    &["host", "tag"],
    &["host", "tag", "devbox"],
    &["host", "tag", "devbox", "work", "client acme"],
    &["host", "explode"],
    &["host", "ports"],
    &["host", "forward", "3000"],
    &["host", "usage"],
    // ports
    &["ports"],
    &["ports", "list"],
    &["ports", "list", "--json", "--window", "4"],
    &["ports", "forward", "3000"],
    &["ports", "unforward", "3000"],
    &["ports", "open", "3000"],
    &["ports", "open", "65535"],
    &["ports", "open", "0"],
    &["ports", "forward"],
    &["ports", "unforward"],
    &["ports", "open"],
    &["ports", "forward", "65536"],
    &["ports", "forward", "http"],
    &["ports", "forward", "3000", "3001"],
    &["ports", "select", "devbox"],
    &["ports", "mute"],
    // chat
    &["chat"],
    &["chat", "status"],
    &["chat", "list"],
    &["chat", "--json", "--window", "7"],
    &["chat", "show"],
    &["chat", "hide"],
    &["chat", "toggle"],
    &["chat", "hide", "--pane", "w1:p2"],
    &["chat", "--pane", "w1:p2", "show"],
    &["chat", "show", "--pane"],
    &["chat", "show", "--pane", "a", "--pane", "b"],
    &["chat", "show", "extra"],
    &["chat", "status", "--pane", "w1:p2"],
    &["chat", "new", "claude"],
    &["chat", "new", "codex"],
    &["chat", "new", "copilot"],
    &["chat", "new", "gemini"],
    &["chat", "new", "opencode"],
    &["chat", "new", "pi"],
    &["chat", "new", "--pane", "w1:p2", "codex"],
    &["chat", "new", "codex", "--pane", "w1:p2", "--json"],
    &["chat", "new"],
    &["chat", "new", "cursor"],
    &["chat", "new", "Claude"],
    &["chat", "new", "claude", "codex"],
    &["chat", "new", "--pane"],
    &["chat", "prompt", "hello"],
    // notes
    &["notes"],
    &["notes", "list"],
    &["notes", "get"],
    &["notes", "clear"],
    &["notes", "clear", "--json", "--window", "2"],
    &["notes", "set"],
    &["notes", "set", "waiting", "on", "review"],
    &["notes", "set", "waiting on review", "--json"],
    &["notes", "set", ""],
    &["notes", "set", "--", "--json"],
    &["notes", "clear", "extra"],
    &["notes", "list", "extra"],
    &["notes", "explode"],
    // worktree
    &["worktree"],
    &["worktree", "status"],
    &["worktree", "mute"],
    &["worktree", "unmute"],
    &["worktree", "mute", "--json", "--window", "9"],
    &["worktree", "silence"],
    &["worktree", "mute", "extra"],
];

/// Note text built to sit at, and just past, the parser's bound, with the
/// name the fixture shows in its place.
fn long_notes() -> Vec<(String, String)> {
    [
        ("x", NOTE_LIMIT),
        ("x", NOTE_LIMIT + 1),
        // Two bytes each: the bound counts bytes, not characters.
        ("é", NOTE_LIMIT / 2),
        ("é", NOTE_LIMIT / 2 + 1),
    ]
    .into_iter()
    .map(|(unit, count)| (unit.repeat(count), format!("<{unit} * {count}>")))
    .collect()
}

/// Runs the client function of a parsed command against [`record`].
fn exchange(command: ControlCommand, context: &Context) {
    // Only the request matters: `Done` is not every command's answer.
    let _ = match command {
        ControlCommand::App(command) => app::app(command, context),
        ControlCommand::Window(command) => app::window(command, context),
        ControlCommand::Ui(command) => app::ui(command, context),
        ControlCommand::Tab(command) => tabs::tab(command, context),
        ControlCommand::Host(command) => hosts::host(command, context),
        ControlCommand::Ports(command) => hosts::ports(command, context),
        ControlCommand::Chat(command) => chat::chat(command, context),
        ControlCommand::Notes(command) => work::notes(command, context),
        ControlCommand::Worktree(command) => work::worktree(command, context),
        command => panic!("{command:?} is not one of the recorded families"),
    };
}

/// One command line's fixture line: what it sends, or how it is refused.
fn outcome(args: &[&str]) -> String {
    let invocation = match LaunchOptions::parse(args.iter().copied()) {
        Ok(LaunchOptions {
            mode: LaunchMode::Control(invocation),
            ..
        }) => invocation,
        Ok(options) => panic!("{args:?} parsed as {:?}", options.mode),
        Err(error) => return format!("error {error:?}: {error}"),
    };
    let context = Context {
        transport: record,
        ..Context::new(invocation.json, invocation.window)
    };
    SENT.with_borrow_mut(Vec::clear);
    exchange(invocation.command, &context);
    let sent = SENT.take();
    let [request] = sent.as_slice() else {
        panic!("{args:?} sent {} requests", sent.len());
    };
    let mut request = serde_json::to_value(request).unwrap();
    // The caller's pane, workspace, and daemon come from the environment;
    // the window is the only part of it a command line decides.
    let caller = request
        .as_object_mut()
        .and_then(|request| request.remove("caller"))
        .unwrap();
    let window = caller
        .get("window")
        .map_or_else(|| "none".to_owned(), |window| window.to_string());
    format!("json={} window={window} {request}", invocation.json)
}

/// A command line as a shell would need it typed.
fn shown(args: &[&str]) -> String {
    let words: Vec<String> = args
        .iter()
        .map(|arg| {
            if arg.is_empty() || arg.contains(char::is_whitespace) {
                format!("{arg:?}")
            } else {
                (*arg).to_owned()
            }
        })
        .collect();
    words.join(" ")
}

fn recorded() -> String {
    let notes = long_notes();
    let mut rows: Vec<Vec<&str>> = ROWS.iter().map(|row| row.to_vec()).collect();
    rows.extend(notes.iter().map(|(text, _)| vec!["notes", "set", text]));
    let mut text = String::new();
    for row in rows {
        let mut entry = format!("$ {}\n{}\n", shown(&row), outcome(&row));
        for (long, name) in &notes {
            entry = entry.replace(long.as_str(), name);
        }
        text.push_str(&entry);
    }
    text
}

#[test]
fn every_family_command_sends_the_recorded_request() {
    assert_eq!(recorded(), include_str!("wire.txt"));
}
