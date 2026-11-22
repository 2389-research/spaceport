// ABOUTME: Exposes the shared Spaceport idle-age bands to the Swift Agents screen.
// ABOUTME: Classifies archived observations using the time they were last confirmed.
pub use spaceport_client::agent_idle::IdleAgeGroup;

#[uniffi::remote(Enum)]
enum IdleAgeGroup {
    Recent,
    OneToFiveHours,
    FiveHoursOrMore,
    Unplaced,
}

#[uniffi::export]
pub fn idle_age_group(
    first_observed_ms: Option<u64>,
    onset_observed: bool,
    confirmed_at_ms: u64,
) -> IdleAgeGroup {
    first_observed_ms.map_or(IdleAgeGroup::Unplaced, |first_observed_ms| {
        spaceport_client::agent_idle::IdleObservation {
            first_observed_ms,
            onset_observed,
        }
        .group(confirmed_at_ms)
    })
}
