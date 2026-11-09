#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn typed_line_runs_only_the_reviewed_script() {
    let line = command_line();
    let quoted = line
        .strip_prefix("sh -ec '")
        .and_then(|rest| rest.strip_suffix('\''))
        .unwrap();
    assert!(!quoted.contains(['\'', '\\', '\n']), "{line}");
    assert_eq!(quoted, r#"eval "$HERDR_WORKTREE_SCRIPT""#);
    assert!(!line.contains("worktree remove"));
}

#[test]
fn tab_params_carry_the_script_and_paths_as_environment() {
    let checkout = Checkout {
        path: "/w/feat".into(),
        root: Some("/r".into()),
    };
    assert_eq!(
        tab_params("w2", &checkout, ScriptKind::Setup, "npm ci"),
        json!({"workspace_id":"w2","cwd":"/w/feat","label":"setup","focus":true,
            "env":{"HERDR_WORKTREE_SCRIPT":"npm ci","HERDR_WORKTREE_PATH":"/w/feat","HERDR_ROOT_PATH":"/r"}})
    );
    let params = tab_params(
        "w2",
        &Checkout {
            root: None,
            ..checkout
        },
        ScriptKind::Run,
        "make",
    );
    assert_eq!(params["label"], "run");
    assert!(params["env"].get(ROOT_ENV).is_none());
}

#[test]
fn locates_a_workspace_checkout_and_its_main_checkout() {
    let response = json!({"result":{"type":"worktree_list","worktrees":[
        {"path":"/r","is_linked_worktree":false,"is_bare":false,"open_workspace_id":"w1"},
        {"path":"/w/feat","is_linked_worktree":true,"is_bare":false,"open_workspace_id":"w2"},
        {"path":"/w/other","is_linked_worktree":true,"is_bare":false}
    ]}});
    assert_eq!(
        locate(&response, "w2").unwrap(),
        Checkout {
            path: "/w/feat".into(),
            root: Some("/r".into())
        }
    );
    assert_eq!(locate(&response, "w1").unwrap().path, "/r");
    assert!(matches!(
        locate(&response, "w9"),
        Err(crate::Error::WorktreeScripts(Error::Checkout))
    ));
    assert!(matches!(
        locate(&json!({"error":{"code":"x","message":"no"}}), "w2"),
        Err(crate::Error::Daemon(crate::daemon::Error::Response(_)))
    ));
    // Two entries claiming the workspace identify nothing.
    let ambiguous = json!({"result":{"type":"worktree_list","worktrees":[
        {"path":"/a","is_linked_worktree":true,"is_bare":false,"open_workspace_id":"w2"},
        {"path":"/b","is_linked_worktree":true,"is_bare":false,"open_workspace_id":"w2"}
    ]}});
    assert!(locate(&ambiguous, "w2").is_err());
    // A bare repository has no main checkout to name.
    let bare = json!({"type":"worktree_list","worktrees":[
        {"path":"/r.git","is_linked_worktree":false,"is_bare":true}
    ]});
    assert_eq!(main_checkout(&bare), None);
}

#[test]
fn created_tabs_name_their_pane() {
    let created = json!({"result":{"type":"tab_created","tab":{"tab_id":"w2:t3"},"root_pane":{"pane_id":"w2:p5"}}});
    assert_eq!(
        created_tab(&created).unwrap(),
        ("w2:t3".to_owned(), "w2:p5".to_owned())
    );
    assert!(matches!(
        created_tab(&json!({"result":{"type":"workspace_created"}})),
        Err(crate::Error::WorktreeScripts(Error::Response))
    ));
    assert!(matches!(
        created_tab(&json!({"error":{"code":"denied","message":"no"}})),
        Err(crate::Error::Daemon(crate::daemon::Error::Response(_)))
    ));
}
