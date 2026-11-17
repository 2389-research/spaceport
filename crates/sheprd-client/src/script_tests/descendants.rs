// ABOUTME: Exercises script deadlines after the direct child has exited.
// ABOUTME: Uses a short-lived descendant that retains the script's output pipes.
use super::*;

#[test]
fn an_exited_parent_does_not_disable_pipe_deadlines() {
    let started = Instant::now();
    let (result, _) = local(
        "sleep 2 &",
        &[],
        ScriptLimits {
            output: 1024,
            idle: Duration::from_millis(100),
        },
    );
    assert!(matches!(result, Err(Error::ScriptTimeout)), "{result:?}");
    assert!(started.elapsed() < Duration::from_secs(1));
}
