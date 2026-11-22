//! Which forward a click on a remote port may reuse, given its state and
//! whether its SSH still runs. App tests cannot start a real SSH child, so
//! the dead-master and missing-handle cases are checked here directly.

use super::*;

const LISTENING: State = State::Listening { local_port: 14_000 };
const HANDLELESS_IN_TESTS: bool = true;
const HANDLELESS_IN_PRODUCTION: bool = false;

fn ended() -> State {
    State::Ended("gone".into())
}

#[test]
fn a_starting_forward_is_reused_whatever_its_master() {
    for alive in [Some(true), Some(false), None] {
        for handleless in [HANDLELESS_IN_TESTS, HANDLELESS_IN_PRODUCTION] {
            assert_eq!(
                reuse_of(&State::Starting, alive, handleless),
                Some(Reuse::Starting),
                "{alive:?} {handleless}"
            );
        }
    }
}

#[test]
fn a_listening_forward_is_reused_only_while_its_master_runs() {
    for handleless in [HANDLELESS_IN_TESTS, HANDLELESS_IN_PRODUCTION] {
        assert_eq!(
            reuse_of(&LISTENING, Some(true), handleless),
            Some(Reuse::Listening { local_port: 14_000 })
        );
        // SSH exited before its report arrived: tunnel afresh.
        assert_eq!(reuse_of(&LISTENING, Some(false), handleless), None);
    }
}

#[test]
fn a_listening_forward_without_a_handle_is_reused_only_in_tests() {
    assert_eq!(
        reuse_of(&LISTENING, None, HANDLELESS_IN_TESTS),
        Some(Reuse::Listening { local_port: 14_000 })
    );
    assert_eq!(reuse_of(&LISTENING, None, HANDLELESS_IN_PRODUCTION), None);
}

#[test]
fn an_ended_forward_is_never_reused() {
    for alive in [Some(true), Some(false), None] {
        for handleless in [HANDLELESS_IN_TESTS, HANDLELESS_IN_PRODUCTION] {
            assert_eq!(reuse_of(&ended(), alive, handleless), None);
        }
    }
}
