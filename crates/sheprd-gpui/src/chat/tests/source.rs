// ABOUTME: Pins what a chat target's source decides: its identity, how it is dialled, and who takes input.
// ABOUTME: Covers a program at its endpoint and an observed CLI session, on each route.
use super::overlay::{attach_with, open};
use super::*;
use crate::{
    chat::{
        ChatInput, Chats, Program as Helper, chats, discovery, existing::Binding, link::Dial,
        link::Link,
    },
    config::chat::TranscriptMirror,
    control::Response,
};
use gpui::TestAppContext;
use sheprd_chat_protocol::EndpointId;
use sheprd_client::{ConnectTarget, protocol::ClientShellSnapshot};
use sheprd_control_protocol::protocol::chat::{InputMode, Reply};
use std::{
    collections::HashSet,
    hash::{BuildHasher, RandomState},
};

const ENDPOINT: &str = "fedcba9876543210fedcba9876543210";
const SESSION: &str = "01234567-89ab-4cde-8123-456789abcdef";
/// The endpoint id that spells [`SESSION`].
const SESSION_AS_ENDPOINT: &str = "0123456789ab4cde8123456789abcdef";

fn binding(pane: &str) -> Binding {
    Binding {
        pane_id: pane.into(),
        agent: sheprd_chat_protocol::Agent::Codex,
        session_id: SESSION.into(),
        herdr_session: Some("test-session".into()),
        pid: 21,
        shell_pid: 20,
        started_ticks: 123,
    }
}

/// A target for the program listening at `endpoint` in `pane`.
fn program(pane: &str, endpoint: &str, route: Route) -> Target {
    Target {
        pane_id: pane.into(),
        route,
        source: Source::Program(endpoint.parse::<EndpointId>().unwrap()),
        surface: Surface::Herdr,
    }
}

/// A target observing `binding`'s CLI session, made by hand.
fn observed(binding: Binding, route: Route) -> Target {
    Target {
        pane_id: binding.pane_id.clone(),
        route,
        source: Source::Observed(binding),
        surface: Surface::Herdr,
    }
}

/// Makes `target` observe `binding`, as a pane whose agent was found does.
fn observe(target: &mut Target, binding: Binding) {
    target.source = Source::Observed(binding);
}

/// Codex panes, each with the endpoint token its chat program reported.
fn panes(agents: &[(&str, Option<&str>)]) -> ClientShellSnapshot {
    let mut snapshot = crate::sidebar::layout_tests::snapshot(1);
    let template = snapshot.agents[0].clone();
    snapshot.agents = agents
        .iter()
        .map(|(pane, token)| {
            let mut agent = template.clone();
            agent.pane_id = (*pane).into();
            agent.agent = Some("codex".into());
            agent.tokens = token
                .iter()
                .map(|token| (sheprd_chat_protocol::TOKEN.to_owned(), (*token).to_owned()))
                .collect();
            agent
        })
        .collect();
    snapshot
}

/// The targets the window's discovery makes on this machine.
fn discovered(snapshot: &ClientShellSnapshot, bindings: &[Binding]) -> Vec<Target> {
    discovery::with_existing(
        discovery::targets(snapshot, Some(&Route::Local)),
        snapshot,
        Some(&Route::Local),
        bindings,
    )
}

fn same_key(left: &Target, right: &Target) -> bool {
    let hasher = RandomState::new();
    left == right && hasher.hash_one(left) == hasher.hash_one(right)
}

/// Reconciles `targets`, counting the connections started.
fn reconcile(chats: &mut Chats, listener: &Program, targets: Vec<Target>, started: &mut usize) {
    let path = listener.path.clone();
    chats.reconcile(targets, Instant::now(), &mut |_, since, resend| {
        *started += 1;
        Link::start(path.clone(), since, resend)
    });
}

#[test]
fn two_panes_observing_one_session_are_distinct_targets() {
    let snapshot = panes(&[("a", None), ("b", None)]);
    let targets = discovered(&snapshot, &[binding("a"), binding("b")]);
    let ids: Vec<_> = targets.iter().map(|t| t.pane_id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
    assert_ne!(targets[0], targets[1]);
    assert_eq!(targets.iter().collect::<HashSet<_>>().len(), 2);
    // Discovery's targets are the ones made by hand.
    assert!(same_key(&targets[0], &observed(binding("a"), Route::Local)));
    assert!(same_key(&targets[1], &observed(binding("b"), Route::Local)));
}

#[test]
fn a_rediscovered_pane_is_the_same_target_and_keeps_its_connection() {
    let snapshot = panes(&[("a", None), ("b", None)]);
    let bindings = [binding("a"), binding("b")];
    let first = discovered(&snapshot, &bindings);
    let again = discovered(&snapshot, &bindings);
    assert!(same_key(&first[0], &again[0]));
    assert!(same_key(&first[1], &again[1]));

    let listener = Program::new();
    let mut chats = Chats::default();
    let mut started = 0;
    reconcile(&mut chats, &listener, first.clone(), &mut started);
    assert_eq!(started, 2);
    assert!(chats.set_raw("a", true));
    reconcile(&mut chats, &listener, again, &mut started);
    assert_eq!(started, 2, "the same targets start no new connection");
    assert!(chats.raw("a"), "and keep the user's choice of view");

    // Another process in pane `a` is another target; `b` is left alone.
    let mut replaced = bindings.clone();
    replaced[0].pid += 1;
    let changed = discovered(&snapshot, &replaced);
    assert_ne!(changed[0], first[0]);
    assert!(same_key(&changed[1], &first[1]));
    reconcile(&mut chats, &listener, changed, &mut started);
    assert_eq!(started, 3);
    assert!(!chats.raw("a"));
}

#[test]
fn an_observed_target_and_a_program_target_of_one_pane_are_distinct() {
    // Even a program whose endpoint spells the observed session's id.
    for endpoint in [ENDPOINT, SESSION_AS_ENDPOINT] {
        let program_target = discovered(&panes(&[("a", Some(endpoint))]), &[]);
        let observed_target = discovered(&panes(&[("a", None)]), &[binding("a")]);
        assert_eq!(
            program_target,
            [program("a", endpoint, Route::Local)],
            "{endpoint}"
        );
        assert_eq!(observed_target.len(), 1);
        assert_ne!(program_target[0], observed_target[0], "{endpoint}");
        let both: HashSet<_> = program_target.iter().chain(&observed_target).collect();
        assert_eq!(both.len(), 2, "{endpoint}");

        let listener = Program::new();
        let mut chats = Chats::default();
        let mut started = 0;
        reconcile(&mut chats, &listener, program_target, &mut started);
        assert!(chats.set_raw("a", true));
        reconcile(&mut chats, &listener, observed_target, &mut started);
        assert_eq!(started, 2, "{endpoint}: the pane's chat is replaced");
        assert!(!chats.raw("a"), "{endpoint}");
    }
}

#[test]
fn a_real_program_wins_its_pane_over_an_observed_session() {
    let snapshot = panes(&[("a", Some(ENDPOINT))]);
    assert_eq!(
        discovered(&snapshot, &[binding("a")]),
        [program("a", ENDPOINT, Route::Local)]
    );
}

#[test]
fn a_session_that_is_not_a_uuid_is_not_observed() {
    let mut binding = binding("a");
    binding.session_id = "not-a-session-uuid".into();
    assert!(discovered(&panes(&[("a", None)]), &[binding]).is_empty());
}

#[test]
fn a_mirror_is_a_program_at_its_endpoint() {
    let mirrors = [TranscriptMirror {
        pane: "a".into(),
        endpoint: ENDPOINT.into(),
        host: None,
    }];
    let snapshot = panes(&[("a", None)]);
    let targets =
        discovery::with_mirrors(Vec::new(), &snapshot, Some(&Route::Local), &mirrors, false);
    assert_eq!(targets, [program("a", ENDPOINT, Route::Local)]);
}

/// The program and arguments of the child `dial` would start.
fn child(dial: Result<Dial>) -> (String, Vec<String>) {
    match dial {
        Ok(Dial::Process(command)) => (
            command.get_program().to_string_lossy().into_owned(),
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        ),
        Ok(Dial::Socket(path)) => panic!("dialled the socket {path:?}"),
        Err(error) => panic!("{error:?}"),
    }
}

/// The child that runs `script` on `host`.
fn remote(host: &str, script: &str) -> (String, Vec<String>) {
    child(Ok(Dial::Process(
        sheprd_client::script_command(host, script).unwrap(),
    )))
}

#[test]
fn a_local_program_is_dialled_at_its_endpoints_socket() {
    let target = program("a", ENDPOINT, Route::Local);
    let address = sheprd_chat_protocol::endpoint_address(&ENDPOINT.parse().unwrap());
    // The helper is not looked for: the program already runs.
    match (chats::dial(&target, &Helper::Missing), address) {
        (Ok(Dial::Socket(path)), Some(address)) => assert_eq!(path, address),
        (Err(Error::NoEndpointDir), None) => {}
        (Ok(Dial::Socket(path)), None) => panic!("dialled {path:?} with no socket directory"),
        (Ok(Dial::Process(_)), _) => panic!("started a child for a local program"),
        (Err(error), _) => panic!("{error:?}"),
    }
}

#[test]
fn a_remote_program_is_dialled_by_connect_over_ssh() {
    let target = program("a", ENDPOINT, Route::Ssh("host".into()));
    let (ssh, args) = child(chats::dial(&target, &Helper::Missing));
    let script = remote_program_script(&format!("connect {ENDPOINT}"));
    assert_eq!((ssh, args.clone()), remote("host", &script));
    assert!(args.iter().any(|arg| arg == "host"));
    let command = args.last().unwrap();
    assert!(
        command.contains(&format!("exec \"$p\" connect {ENDPOINT};")),
        "{command}"
    );
    assert!(!command.contains("observe"), "{command}");
}

#[test]
fn a_local_session_is_observed_by_the_located_helper() {
    let target = observed(binding("a"), Route::Local);
    let helper = Helper::Found("/opt/spaceport/sheprd-acp".into());
    let (started, args) = child(chats::dial(&target, &helper));
    assert_eq!(started, "/opt/spaceport/sheprd-acp");
    assert_eq!(
        args,
        ["observe", "codex", SESSION, "21", "123", "--pane", "a"]
    );
}

#[test]
fn a_remote_session_is_observed_over_ssh_without_a_local_helper() {
    let target = observed(binding("a"), Route::Ssh("host".into()));
    let (ssh, args) = child(chats::dial(&target, &Helper::Missing));
    let observe = format!("'observe' 'codex' '{SESSION}' '21' '123' '--pane' 'a'");
    let script = remote_program_script(&observe);
    assert_eq!((ssh, args.clone()), remote("host", &script));
    let command = args.last().unwrap();
    assert!(!command.contains("connect"), "{command}");
    assert!(!command.contains(SESSION_AS_ENDPOINT), "{command}");
}

/// What `chat status` says takes the fixture pane's input, beside what the
/// pane itself says.
fn input_modes(
    view: &gpui::Entity<crate::SheprdWindow>,
    cx: &mut gpui::VisualTestContext,
) -> (InputMode, ChatInput) {
    let input = view.read_with(cx, |view, _| view.chats.pane("w1:p1").unwrap().input());
    let Response::Chat {
        reply: Reply::Chats { chats },
    } = cx.cx.update(crate::control::chat_status)
    else {
        panic!("chat status did not list the chats");
    };
    assert_eq!(chats.len(), 1);
    assert_eq!(chats[0].pane_id, "w1:p1");
    (chats[0].input_mode, input)
}

/// The fixture window with its chat attached, the transcript read-only or not.
fn attached(
    read_only: bool,
    cx: &mut TestAppContext,
) -> (
    gpui::Entity<crate::SheprdWindow>,
    Peer,
    &mut gpui::VisualTestContext,
) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let listener = Program::new();
    let mut transcript = snapshot(cursor(1, 0), vec![agent(1, "History")]);
    if let ServerMessage::Snapshot { transcript, .. } = &mut transcript {
        transcript.read_only = read_only;
    }
    let stream = attach_with(&view, &listener, transcript, cx);
    (view, stream, cx)
}

#[gpui::test]
fn chat_status_reports_a_program_chats_composer(cx: &mut TestAppContext) {
    let (view, _stream, cx) = attached(false, cx);
    assert_eq!(
        input_modes(&view, cx),
        (InputMode::Chat, ChatInput::Composer)
    );
}

#[gpui::test]
fn chat_status_reports_a_read_only_mirror(cx: &mut TestAppContext) {
    let (view, _stream, cx) = attached(true, cx);
    assert_eq!(
        input_modes(&view, cx),
        (InputMode::ReadOnly, ChatInput::ReadOnly)
    );
}

#[gpui::test]
fn chat_status_reports_an_observed_chats_terminal(cx: &mut TestAppContext) {
    // An observer's transcript is read-only too; the source still decides.
    for read_only in [true, false] {
        let (view, _stream, cx) = attached(read_only, cx);
        view.update(cx, |view, _| {
            let pane = view.chats.pane_mut("w1:p1").unwrap();
            observe(&mut pane.target, binding("w1:p1"));
        });
        assert_eq!(
            input_modes(&view, cx),
            (InputMode::Terminal, ChatInput::Terminal),
            "read_only: {read_only}"
        );
    }
}
