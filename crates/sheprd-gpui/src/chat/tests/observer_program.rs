//! An existing agent on this machine is observed by the `herdr-acp` the
//! app located, not a path guessed beside the executable.

use super::*;
use crate::chat::{Program, StartError, chats, existing::Binding};
use sheprd_chat_protocol::Agent;

fn observed() -> Target {
    Target {
        pane_id: "w1:p1".into(),
        route: Route::Local,
        source: Source::Observed(Binding {
            pane_id: "w1:p1".into(),
            agent: Agent::Codex,
            session_id: "12345678-1234-4123-8123-123456789abc".into(),
            herdr_session: None,
            pid: 21,
            shell_pid: 20,
            started_ticks: 123,
        }),
        surface: Surface::Herdr,
    }
}

#[test]
fn an_unlocated_observer_program_is_retried_not_permanent() {
    for (program, expected) in [
        (Program::Unknown, StartError::ProgramUnknown),
        (Program::Looking, StartError::ProgramUnknown),
        (Program::Missing, StartError::ProgramMissing),
    ] {
        let Err(error) = chats::connect(&observed(), &program, None, Vec::new()) else {
            panic!("no program to observe with");
        };
        assert!(
            matches!(&error, Error::Start(found) if std::mem::discriminant(found) == std::mem::discriminant(&expected)),
            "{error:?}"
        );
        assert!(!error.permanent());
    }
}
