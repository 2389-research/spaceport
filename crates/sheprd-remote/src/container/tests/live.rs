//! Opt-in: checks that a user container on the engine named by
//! `HERDR_TEST_CONTAINER_ENGINE` (`docker` or `podman`) is left untouched. Run with
//! `cargo test -p sheprd-remote --features container container::tests::live -- --ignored`.

use super::*;

#[test]
#[ignore = "needs a running Docker or Podman engine"]
fn a_container_this_app_did_not_make_is_never_reused_or_deleted() {
    let engine = std::env::var("HERDR_TEST_CONTAINER_ENGINE")
        .ok()
        .and_then(|key| Engine::from_key(&key))
        .unwrap_or_else(|| panic!("set HERDR_TEST_CONTAINER_ENGINE to docker or podman"));
    let settings = Settings {
        engine: Some(engine),
        ..Settings::resolve(&ContainerConfig::default()).unwrap()
    };
    let never = || false;
    let id = crate::cloud::names::random("herdr", 40);
    // The user's own container, which happens to have a generated name.
    engine
        .run(
            "run",
            [
                "run",
                "--detach",
                "--name",
                &id,
                &settings.image,
                "sleep",
                "120",
            ],
            std::time::Duration::from_secs(60),
            &never,
        )
        .unwrap();
    let deleted = machine::delete(engine, &id, &never);
    let state = machine::state(engine, &id, &never);
    let _ = engine.output(
        "rm",
        ["rm", "--force", &id],
        std::time::Duration::from_secs(30),
        &never,
    );
    assert!(matches!(deleted, Err(Error::NotOurs(_))), "{deleted:?}");
    assert_eq!(state.unwrap(), machine::State::Running, "it was left alone");
}
