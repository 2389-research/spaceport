// ABOUTME: Checks each reported agent name maps to one known identity.
// ABOUTME: Keeps the explicit reopen command catalog consistent.
use super::*;

#[test]
fn every_kind_is_listed_once_and_parses_back() {
    for (index, kind) in AgentKind::ALL.iter().enumerate() {
        assert_eq!(AgentKind::parse(kind.name()), Some(*kind));
        assert!(!AgentKind::ALL[..index].contains(kind), "{kind:?}");
    }
}
