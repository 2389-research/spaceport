use crate::{
    Action, AgentFilter, Entry, NavigationTarget,
    agent_filter::headed,
    search::{self, Highlights},
};
use spaceport_client::protocol::AgentStatus;

#[test]
fn a_status_filter_keeps_only_agents_reporting_that_status() {
    let statuses = [
        None,
        Some(AgentStatus::Idle),
        Some(AgentStatus::Working),
        Some(AgentStatus::Blocked),
        Some(AgentStatus::Done),
        Some(AgentStatus::Unknown),
    ];
    for filter in AgentFilter::ALL {
        let kept: Vec<_> = statuses
            .into_iter()
            .filter(|status| filter.accepts(*status))
            .collect();
        let expected = match filter {
            AgentFilter::All => statuses.to_vec(),
            AgentFilter::Blocked => vec![Some(AgentStatus::Blocked)],
            AgentFilter::Working => vec![Some(AgentStatus::Working)],
            AgentFilter::Idle => vec![Some(AgentStatus::Idle)],
            AgentFilter::Done => vec![Some(AgentStatus::Done)],
        };
        assert_eq!(kept, expected, "{filter:?}");
    }
}

#[test]
fn headings_return_above_their_agents_in_the_order_of_each_workspaces_best_agent() {
    // w1 > a, and w2 > tab > b, c.
    let rows = [
        ("w1", None),
        ("a", Some(0)),
        ("w2", None),
        ("tab", Some(2)),
        ("b", Some(3)),
        ("c", Some(3)),
    ];
    let entries: Vec<_> = rows
        .iter()
        .map(|(label, parent)| {
            Entry::new(
                (*label).to_owned(),
                String::new(),
                "",
                Action::Go {
                    endpoint: String::new(),
                    boot: String::new(),
                    target: NavigationTarget::Pane((*label).to_owned()),
                },
                *parent,
            )
        })
        .collect();
    let hit = |index| search::Hit {
        index,
        depth: 0,
        highlights: Highlights::default(),
    };
    // The query ranked c first, then a, then b.
    let shown: Vec<_> = headed(&entries, vec![hit(5), hit(1), hit(4)])
        .into_iter()
        .map(|hit| (rows[hit.index].0, hit.depth))
        .collect();
    assert_eq!(
        shown,
        [
            ("w2", 0),
            ("tab", 1),
            ("b", 2),
            ("c", 2),
            ("w1", 0),
            ("a", 1)
        ]
    );
    assert!(headed(&entries, Vec::new()).is_empty());
}
