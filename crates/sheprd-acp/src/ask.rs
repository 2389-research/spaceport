// ABOUTME: Owns the app-only observation conversation and its exact Pi child.
// ABOUTME: Keeps durable receipts separate from attachment queue acknowledgements.
mod checkpoint;
pub(crate) mod error;
mod profile;
mod runtime;

use error::Error;
use sheprd_chat_protocol::{
    Command, CommandId, EntryKind, Status, TurnEnd,
    ask::{Checkpoint, Outcome, Phase, Request},
};

fn allowed(state: &Checkpoint, command: &Command) -> bool {
    match command {
        Command::Cancel => matches!(state.outcome, Outcome::Answering),
        Command::Prompt { text, attachments } => {
            attachments.is_empty()
                && !text.trim().is_empty()
                && text.len() <= sheprd_chat_protocol::MAX_PROMPT_SIZE
                && matches!(state.transcript.status, Status::Idle)
                && state.transcript.turn.is_none()
                && matches!(
                    state.outcome,
                    Outcome::Ready | Outcome::Completed | Outcome::Stopped | Outcome::Failed
                )
        }
        _ => false,
    }
}
fn continuation(state: &Checkpoint) -> Option<String> {
    let request = state.request.as_ref()?;
    (state.outcome == Outcome::Interrupted
        && request.phase == Phase::Interrupted
        && state.pi_session_file.is_some()
        && state.pi_session.is_some())
    .then(|| request.text.clone())
}
fn reconcile(state: &mut Checkpoint) {
    if let Some(request) = state.request.as_mut()
        && matches!(request.phase, Phase::Intent | Phase::Accepted)
    {
        request.phase = Phase::Unknown;
        state.outcome = Outcome::Unknown;
    }
}
fn finish(state: &mut Checkpoint, end: TurnEnd, quitting: bool) {
    let Some(request) = state.request.as_mut() else {
        return;
    };
    if request.phase == Phase::Stopped {
        state.outcome = Outcome::Stopped;
        return;
    }
    (request.phase, state.outcome) = match end {
        TurnEnd::Cancelled if quitting && request.phase == Phase::Accepted => {
            (Phase::Interrupted, Outcome::Interrupted)
        }
        TurnEnd::Cancelled => (Phase::Unknown, Outcome::Unknown),
        TurnEnd::Failed => (Phase::Completed, Outcome::Failed),
        _ => (Phase::Completed, Outcome::Completed),
    };
}

fn stop(state: &mut Checkpoint) {
    if let Some(request) = &mut state.request {
        request.phase = Phase::Stopped;
    }
    if state.outcome != Outcome::Unknown {
        state.outcome = Outcome::Stopped;
    }
}

pub(crate) fn run(options: crate::cli::AskOptions) -> anyhow::Result<()> {
    let store = checkpoint::Store::open(&options.state_dir)?;
    let cwd = options.state_dir.canonicalize()?;
    let mut state = match store.load()? {
        Some(mut state) => {
            if state.version != sheprd_chat_protocol::ask::VERSION
                || state.conversation_id != options.conversation
                || state.endpoint != options.endpoint.as_str()
            {
                return Err(Error::InvalidState.into());
            }
            reconcile(&mut state);

            state
        }
        None => Checkpoint {
            version: sheprd_chat_protocol::ask::VERSION,
            conversation_id: options.conversation,
            helper_run_id: uuid::Uuid::new_v4(),
            endpoint: options.endpoint.to_string(),
            pi_session: None,
            pi_session_file: None,
            request: None,
            transcript: sheprd_chat_protocol::Transcript::new("Pi"),
            outcome: Outcome::Ready,
        },
    };
    let stop_marker = options.state_dir.join(sheprd_chat_protocol::ask::STOP_FILE);
    if stop_marker.exists() {
        stop(&mut state);
        store.save(&state)?;
        std::fs::remove_file(stop_marker)?;
    }
    if state.outcome == Outcome::Unknown {
        store.save(&state)?;
        return Err(Error::UnknownOutcome.into());
    }
    state.helper_run_id = uuid::Uuid::new_v4();
    if state.pi_session.is_some() != state.pi_session_file.is_some() {
        return Err(Error::InvalidState.into());
    }
    validate_session(&mut state, &cwd)?;
    let search = crate::agents::search_path(
        std::env::var_os("PATH").as_deref(),
        std::env::home_dir().as_deref(),
    );
    let program = crate::agents::find_program(crate::pi::PROGRAM, Some(&search))
        .ok_or(Error::MissingPi)?
        .into_os_string();
    profile::qualify(&program, &cwd)?;
    std::fs::create_dir_all(cwd.join("sessions"))?;
    let broker = serde_json::to_string(&options.broker)?;
    let extension =
        include_str!("ask/observation.ts").replace("__SPACEPORT_BROKER_JSON__", &broker);
    store.replace("observation.ts", extension.as_bytes())?;
    let args = profile::args(
        &cwd,
        &cwd.join("observation.ts"),
        state.pi_session_file.as_deref(),
    );
    let spawned = crate::process::spawn(&program, &args, &[], &cwd)?;
    let owned = Owned(spawned.process.clone());
    let result = runtime::run(options, &store, &mut state, spawned);
    drop(owned);
    result
}

fn validate_session(state: &mut Checkpoint, directory: &std::path::Path) -> Result<(), Error> {
    let Some(file) = &state.pi_session_file else {
        return Ok(());
    };
    let resolved = match file.canonicalize() {
        Ok(path) => path,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                && state.request.is_none()
                && state
                    .transcript
                    .entries
                    .iter()
                    .all(|entry| matches!(entry.kind, EntryKind::Notice { .. })) =>
        {
            // Pi can reserve a session path before an unused session writes its file.
            state.pi_session = None;
            state.pi_session_file = None;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    if !resolved.starts_with(directory.join("sessions").canonicalize()?) {
        return Err(Error::InvalidState);
    }
    Ok(())
}

struct Owned(crate::process::AgentProcess);
impl Drop for Owned {
    fn drop(&mut self) {
        self.0.stop();
    }
}

#[cfg(test)]
mod tests;
