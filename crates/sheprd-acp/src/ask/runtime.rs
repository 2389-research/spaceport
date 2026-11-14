// ABOUTME: Runs one bounded app-only session loop and persists each observed transition.
// ABOUTME: Checkpoints before dispatch and Stop, and never replays an uncertain submission.
use super::*;
use crate::{
    cli::AskOptions,
    harness::Pipes,
    session::{Config, ConnId, Flow, Input, Session},
};
use crossbeam_channel::{RecvTimeoutError, bounded};
use sheprd_chat_protocol::ask::QUIT_FILE;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

pub(super) fn run(
    options: AskOptions,
    store: &checkpoint::Store,
    state: &mut Checkpoint,
    spawned: crate::process::Spawned,
) -> anyhow::Result<()> {
    let (inputs, receiver) = bounded(1024);
    let deliver = inputs.clone();
    let resume = continuation(state);
    let outbox = crate::pi::restricted_reach(resume.is_some()).connect(
        Pipes {
            input: Box::new(spawned.input),
            output: Box::new(spawned.output),
        },
        Box::new(move |inbound| deliver.send(Input::Agent(inbound)).is_ok()),
    )?;
    let role_process = spawned.process.clone();
    let process = spawned.process;
    let waiter = inputs.clone();
    thread::Builder::new()
        .name("ask-pi-wait".into())
        .spawn(move || {
            let code = process.wait().ok().and_then(|status| status.code());
            let _ = waiter.send(Input::AgentExited {
                code,
                stderr: process.stderr_tail(),
            });
        })?;
    let quit = options.state_dir.join(QUIT_FILE);
    let address = sheprd_chat_protocol::endpoint_address(&options.endpoint)
        .ok_or(crate::Error::NoEndpointDir)?;
    let _server = crate::attach::Server::listen(&address, inputs)?;
    let config = Config {
        agent: "Pi".into(),
        cwd: options.state_dir.clone(),
        exit_with_agent: true,
        history: Some(state.transcript.clone()),
        load: state.pi_session.clone(),
        login_hint: Some("run Pi and use its existing sign-in".into()),
        pane: None,
        require_load: state.pi_session.is_some(),
    };
    let mut session = Session::new(config, outbox, |_| {});
    state.transcript = session.transcript().clone();
    // A fresh helper receipt is visible only after the attach listener exists.
    store.save(state)?;
    session.start();
    let mut continuation = resume;
    let stop_marker = options.state_dir.join(sheprd_chat_protocol::ask::STOP_FILE);
    let mut quitting = None;
    let startup = Instant::now();
    let mut role_receipt = false;
    loop {
        if !role_receipt {
            // Pi routes extension stdout to its stderr; only its bounded owned tail is inspected.
            if let Some(receipt) = role_process
                .stderr_tail()
                .iter()
                .rev()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .find(|value| {
                    value.get("type").and_then(serde_json::Value::as_str)
                        == Some("spaceport_ask_ready")
                })
            {
                session.confirm_role(&receipt)?;
                role_receipt = true;
            }
        }

        if stop_marker.exists() {
            stop(state);
            continuation = None;
            store.save(state)?;
            std::fs::remove_file(&stop_marker)?;
            if session.transcript().turn.is_some() {
                session.handle(Input::Command {
                    conn: ConnId(0),
                    id: CommandId::random(),
                    command: Command::Cancel,
                });
            }
        }

        if matches!(session.transcript().status, Status::Starting)
            && startup.elapsed() >= Duration::from_secs(5)
        {
            return Err(Error::RoleNotReady.into());
        }
        if quitting.is_none() && quit.exists() {
            std::fs::remove_file(&quit)?;
            quitting = Some(Instant::now());
            store.save(state)?;
            if session.transcript().turn.is_some() {
                session.handle(Input::Command {
                    conn: ConnId(0),
                    id: CommandId::random(),
                    command: Command::Cancel,
                });
            }
        }
        if let Some(start) = quitting {
            if session.transcript().turn.is_none() {
                store.save(state)?;
                return Ok(());
            }
            if start.elapsed() >= Duration::from_secs(3) {
                if state.outcome != Outcome::Stopped {
                    state.outcome = Outcome::Unknown;
                    if let Some(request) = &mut state.request {
                        request.phase = Phase::Unknown;
                    }
                }
                store.save(state)?;
                return Ok(());
            }
        }
        if continuation.is_some()
            && matches!(session.transcript().status, Status::Idle)
            && quitting.is_none()
        {
            let text = continuation.take().ok_or(Error::InvalidState)?;
            if let Some(request) = &mut state.request {
                request.phase = Phase::Intent;
            }
            state.outcome = Outcome::Answering;
            store.save(state)?;
            session.resume_prompt(&text);
        }
        let input = match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(input) => input,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        };
        let mut cancel_started = false;
        if let Input::Agent(crate::rpc::Inbound::Notification { method, params }) = &input {
            if method == "spaceport/ask_state" {
                metadata(state, params, &options.state_dir)?;
            }
            if method == "spaceport/ask_accepted"
                && params.get("request").and_then(serde_json::Value::as_i64)
                    == session.active_request()
                && let Some(request) = &mut state.request
            {
                if request.phase == Phase::Intent {
                    request.phase = Phase::Accepted;
                }
                cancel_started = quitting.is_some() || request.phase == Phase::Stopped;
            }
        }
        let input = match input {
            Input::Command { conn, id, command } => {
                if quitting.is_some()
                    || !allowed(state, &command)
                    || state
                        .request
                        .as_ref()
                        .is_some_and(|request| request.id == id)
                {
                    Input::Unsupported { conn, id }
                } else {
                    match &command {
                        Command::Prompt { text, .. } => {
                            state.request = Some(Request {
                                id,
                                text: text.clone(),
                                phase: Phase::Intent,
                            });
                            state.outcome = Outcome::Answering;
                        }
                        Command::Cancel => {
                            if let Some(request) = &mut state.request {
                                request.phase = Phase::Stopped;
                            }
                            state.outcome = Outcome::Stopped;
                        }
                        _ => {}
                    }
                    store.save(state)?;
                    Input::Command { conn, id, command }
                }
            }
            other => other,
        };
        let before = session.transcript().turn.is_some();
        let exited = matches!(input, Input::AgentExited { .. });
        let flow = session.handle(input);
        if cancel_started && session.transcript().turn.is_some() {
            // An earlier abort can precede Pi's start; cancel the accepted turn too.
            store.save(state)?;
            session.handle(Input::Command {
                conn: ConnId(0),
                id: CommandId::random(),
                command: Command::Cancel,
            });
        }
        state.transcript = session.transcript().clone();
        if before
            && state.transcript.turn.is_none()
            && !exited
            && let Some(end) = state
                .transcript
                .entries
                .iter()
                .rev()
                .find_map(|entry| match &entry.kind {
                    EntryKind::Turn(summary) => Some(summary.end),
                    _ => None,
                })
        {
            finish(state, end, quitting.is_some());
        }
        if exited && state.outcome == Outcome::Answering {
            state.outcome = Outcome::Unknown;
            if let Some(request) = &mut state.request {
                request.phase = Phase::Unknown;
            }
        }
        store.save(state)?;
        if flow == Flow::Exit {
            return Ok(());
        }
    }
}

fn metadata(state: &mut Checkpoint, params: &serde_json::Value, dir: &Path) -> Result<(), Error> {
    let Some(id) = params.get("sessionId").and_then(serde_json::Value::as_str) else {
        return Err(Error::InvalidState);
    };
    if state.pi_session.as_ref().is_some_and(|known| known != id) {
        return Err(Error::InvalidState);
    }
    let Some(file) = params
        .get("sessionFile")
        .and_then(serde_json::Value::as_str)
    else {
        return Err(Error::InvalidState);
    };
    let file = std::path::PathBuf::from(file);
    let parent = file.parent().ok_or(Error::InvalidState)?.canonicalize()?;
    let sessions = dir.join("sessions").canonicalize()?;
    if parent != sessions {
        return Err(Error::InvalidState);
    }
    state.pi_session = Some(id.to_owned());
    state.pi_session_file = Some(file);
    Ok(())
}
