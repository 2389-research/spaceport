// ABOUTME: Checks the one definition of "the hub holds this exact live Core".
// ABOUTME: Identity is compared before liveness, so `Lost` names the right Core.

use super::*;
use crate::core_terminals::tests::scripted_core;

fn refuse(_: Start) -> Result<CoreBackend, Error> {
    Err(Error::NotInstalled)
}

#[test]
fn a_hub_without_a_backend_reports_it_absent() {
    let hub = opted_in_hub(refuse);
    assert_eq!(hub.backend_for("core-a").err(), Some(Mismatch::Absent));
}

#[test]
fn only_the_live_connection_to_the_named_core_is_returned() {
    let mut hub = opted_in_hub(refuse);
    let (backend, peer) = scripted_core::connect("core-a");
    hub.set_backend_for_test(backend.clone());
    assert!(hub.backend_for("core-a").is_ok());
    assert_eq!(hub.backend_for("core-b").err(), Some(Mismatch::Other));

    scripted_core::lose(&backend, peer);
    assert_eq!(hub.backend_for("core-a").err(), Some(Mismatch::Lost));
    // Another Core stays another Core once its connection drops.
    assert_eq!(hub.backend_for("core-b").err(), Some(Mismatch::Other));
}
