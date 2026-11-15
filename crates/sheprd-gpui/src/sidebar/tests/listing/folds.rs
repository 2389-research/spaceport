// ABOUTME: Pins that each host keeps its own folded repositories.
// ABOUTME: Checks the fold in the expanded list, the rail, and shortcut stepping alike.
use super::*;
use crate::sidebar::layout_tests::REPO_KEY;

/// The folded repository's linked worktrees, which a fold hides.
const CHILDREN: [&str; 2] = ["w4", "w5"];

/// Which hosts hide the repository's linked worktrees, in every listing:
/// the expanded list, the rail, and stepping in each must agree.
fn folded_hosts(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> Vec<String> {
    let mut answers = Vec::new();
    for mode in [Mode::Expanded, Mode::Rail] {
        let listed = listed(view, cx, mode);
        assert_eq!(
            listed.painted_workspaces, listed.stepped_workspaces,
            "{mode:?}"
        );
        answers.push(
            ["local", REMOTE]
                .into_iter()
                .filter(|host| {
                    let shown = |row: &str| listed.painted_workspaces.iter().any(|r| r == row);
                    assert!(shown(&format!("{host}/w3")), "{mode:?}: the parent stays");
                    let children: Vec<bool> = CHILDREN
                        .iter()
                        .map(|child| shown(&format!("{host}/{child}")))
                        .collect();
                    assert_eq!(children[0], children[1], "{mode:?}: children fold together");
                    !children[0]
                })
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(answers[0], answers[1], "the rail and the list disagree");
    answers.remove(0)
}

#[gpui::test]
fn each_host_keeps_its_own_repository_folds(cx: &mut TestAppContext) {
    let (view, cx) = two_hosts(cx);
    let key = REPO_KEY.to_owned();
    let toggle = |host: &str, cx: &mut VisualTestContext| {
        view.update(cx, |view, cx| view.toggle_repository(host, &key, cx));
    };
    assert!(folded_hosts(&view, cx).is_empty());

    toggle("local", cx);
    assert_eq!(folded_hosts(&view, cx), ["local"]);
    toggle(REMOTE, cx);
    assert_eq!(folded_hosts(&view, cx), ["local", REMOTE]);
    toggle("local", cx);
    assert_eq!(folded_hosts(&view, cx), [REMOTE]);

    // The folds stay with their hosts when the selection moves.
    view.update(cx, |view, cx| {
        view.selected_endpoint = 1;
        view.live = view.endpoints[1].live.clone();
        cx.notify();
    });
    assert_eq!(folded_hosts(&view, cx), [REMOTE]);
    toggle("local", cx);
    assert_eq!(folded_hosts(&view, cx), ["local", REMOTE]);
    toggle(REMOTE, cx);
    assert_eq!(folded_hosts(&view, cx), ["local"]);
    view.update(cx, |view, cx| {
        view.selected_endpoint = 0;
        view.live = view.endpoints[0].live.clone();
        cx.notify();
    });
    assert_eq!(folded_hosts(&view, cx), ["local"]);
}
