// ABOUTME: Refuses moving-mark profile claims when the app applies reduced motion.
// ABOUTME: Allows an idle baseline while preserving the configured accessibility policy.
use super::*;

#[test]
fn working_profile_refuses_reduced_motion() -> Result<()> {
    let (mut observation, targets) = ready(true)?;
    observation.reduce_motion = true;
    assert!(workload(&targets).populated(&observation));
    assert!(workload(&targets).validate(&observation, true).is_err());
    Ok(())
}

#[test]
fn idle_profile_accepts_reduced_motion() -> Result<()> {
    let (mut observation, targets) = ready(false)?;
    observation.reduce_motion = true;
    workload(&targets).validate(&observation, false)
}
