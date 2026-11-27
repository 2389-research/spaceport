// ABOUTME: Pins which saved tabs load, which files are refused, and the bytes written back.
// ABOUTME: Drops retired tab kinds and refuses kinds no build wrote.
use super::*;

/// An absolute path on the platform running the test, as JSON text.
fn absolute(tail: &str) -> String {
    let root = if cfg!(windows) { r"C:\srv" } else { "/srv" };
    serde_json::to_string(&Path::new(root).join(tail)).unwrap()
}

const CORE_SCOPE: &str =
    r#"{"kind":"core","hub_core_id":"hub-a","machine_id":null,"document_name":"spaceport"}"#;
const CORE_GROUP: &str = "00000000-0000-0000-0000-000000000047";

/// One tab of a saved file: `scope` and `location` are JSON, `rest` is any
/// further members, each starting with a comma.
fn tab(id: u64, scope: &str, workspace: &str, location: &str, rest: &str) -> String {
    format!(
        r#"{{"id":{id},"scope":{scope},"workspace_id":"{workspace}","location":{location},"title":"T{id}"{rest}}}"#
    )
}

fn legacy(id: u64, location: &str, rest: &str) -> String {
    tab(id, r#""local""#, "w", location, rest)
}

fn file(tabs: &[String], next: &str) -> Vec<u8> {
    format!(r#"{{"tabs":[{}]{next}}}"#, tabs.join(",")).into_bytes()
}

/// Every kind and place a saved file may hold, out of ID order.
fn every_kind() -> Vec<String> {
    let web = r#"{"kind":"web","url":"https://example.test/docs"}"#;
    let local = format!(
        r#"{{"kind":"local","file":{{"root":{},"entry":"index.html"}}}}"#,
        absolute("site")
    );
    let review = format!(
        r#"{{"kind":"review","checkout":{{"repo_key":{},"branch":"main","checkout":{}}}}}"#,
        absolute("repo/.git"),
        absolute("repo")
    );
    let orchestrator = r#"{"kind":"orchestrator","repo":{"checkout":"/srv/repo"}}"#;
    let code = format!(
        r#"{{"kind":"code","file":{{"path":{},"line":42}}}}"#,
        absolute("repo/main.rs")
    );
    let devices = r#"{"kind":"devices"}"#;
    vec![
        legacy(40, devices, r#","place":"group","favicon":"ignored""#),
        legacy(10, web, ""),
        legacy(3, devices, r#","origin":"w:p1""#),
        legacy(11, &local, ""),
        legacy(5, r#"{"kind":"core_terminal","terminal":"t1"}"#, ""),
        legacy(12, &review, ""),
        legacy(
            6,
            r#"{"kind":"core_terminal","panes":"t2","observe":true}"#,
            "",
        ),
        legacy(13, orchestrator, ""),
        tab(8, CORE_SCOPE, CORE_GROUP, "null", ""),
        legacy(14, &code, ""),
        // A blank tab of a Herdr 1 workspace has nothing to restore.
        legacy(9, "null", ""),
        legacy(20, devices, r#","place":"px0""#),
        legacy(21, devices, r#","place":"code""#),
        legacy(22, "null", r#","place":"code_group""#),
        tab(23, CORE_SCOPE, CORE_GROUP, "null", r#","place":"px0""#),
        legacy(24, &code, r#","place":"code""#),
    ]
}

const SURVIVORS: &str = concat!(
    r#"{"tabs":["#,
    r#"{"id":40,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"T40","origin":null},"#,
    r#"{"id":3,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"T3","origin":"w:p1"},"#,
    r#"{"id":5,"scope":"local","workspace_id":"w","location":{"kind":"core_terminal","panes":"t1"},"title":"T5","origin":null},"#,
    r#"{"id":6,"scope":"local","workspace_id":"w","location":{"kind":"core_terminal","panes":"t2","observe":true},"title":"T6","origin":null},"#,
    r#"{"id":8,"scope":{"kind":"core","hub_core_id":"hub-a","machine_id":null,"document_name":"spaceport"},"workspace_id":"00000000-0000-0000-0000-000000000047","location":null,"title":"T8","origin":null}"#,
);

#[test]
fn a_saved_file_restores_only_retained_tabs_and_reserves_every_id() {
    let store = Store::restored_for_test(&file(&every_kind(), r#","next":7"#)).unwrap();
    assert_eq!(
        store
            .tabs()
            .map(|tab| u64::from(tab.id))
            .collect::<Vec<_>>(),
        [40, 3, 5, 6, 8]
    );
    // One past the largest ID in the file, retained or not.
    assert_eq!(store.next, 41);
    assert_eq!(
        serde_json::to_string(&store.saved()).unwrap(),
        format!(r#"{SURVIVORS}],"next":41}}"#)
    );

    // A file written before `next` existed reserves the same IDs.
    let old = Store::restored_for_test(&file(&every_kind(), "")).unwrap();
    assert_eq!(old.next, 41);
    // A larger saved counter wins: it remembers tabs dropped by an earlier load.
    let later = Store::restored_for_test(&file(&every_kind(), r#","next":100"#)).unwrap();
    assert_eq!(later.next, 100);
}

#[test]
fn a_restored_store_writes_the_same_bytes_and_never_reuses_a_dropped_id() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("browser-tabs.json");
    let mut store = Store::restored_for_test(&file(&every_kind(), r#","next":7"#)).unwrap();
    store.writer =
        Some(spaceport_state_file::Writer::start("browser-tabs-test", path.clone()).unwrap());

    let opened = store
        .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
        .unwrap();
    assert_eq!(u64::from(opened), 41);
    store.writer.take().unwrap().finish();

    let written = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    let devices = r#"{"id":41,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"Devices","origin":null}"#;
    assert_eq!(written, format!("{SURVIVORS},{devices}],\"next\":42}}\n"));
    // What was written loads back to the same tabs.
    let again = Store::restored_for_test(written.as_bytes()).unwrap();
    assert_eq!(again.tabs, store.tabs);
    assert_eq!(again.next, 42);
}

#[test]
fn unknown_kinds_places_and_members_keep_their_outcome() {
    let devices = r#"{"kind":"devices"}"#;
    // An unknown kind or place refuses the whole file, retained tabs included.
    for unknown in [
        legacy(1, r#"{"kind":"hologram"}"#, ""),
        legacy(1, r#"{"kind":"Devices"}"#, ""),
        legacy(1, devices, r#","place":"sidebar""#),
        legacy(1, devices, r#","place":null"#),
    ] {
        let bytes = file(&[legacy(0, devices, ""), unknown], "");
        assert!(
            matches!(Store::restored_for_test(&bytes), Err(Error::Json(_))),
            "{}",
            String::from_utf8_lossy(&bytes)
        );
    }
    // Unknown members of a tab or of its location are ignored.
    let extra = legacy(4, r#"{"kind":"devices","zoom":2}"#, r#","pinned":true"#);
    let store = Store::restored_for_test(&file(&[extra], "")).unwrap();
    assert_eq!(
        serde_json::to_string(&store.saved()).unwrap(),
        r#"{"tabs":[{"id":4,"scope":"local","workspace_id":"w","location":{"kind":"devices"},"title":"T4","origin":null}],"next":5}"#
    );
}

#[test]
fn a_retired_tab_is_dropped_whatever_its_payload() {
    for location in [
        r#"{"kind":"web"}"#,
        r#"{"kind":"web","url":"file:///etc/passwd"}"#,
        r#"{"kind":"local","file":"index.html"}"#,
        r#"{"kind":"review","checkout":{"branch":7}}"#,
        r#"{"kind":"orchestrator","repo":"relative"}"#,
        r#"{"kind":"code","file":{"path":"","line":-1}}"#,
    ] {
        let bytes = file(
            &[
                legacy(0, r#"{"kind":"devices"}"#, ""),
                legacy(1, location, ""),
            ],
            "",
        );
        let store = Store::restored_for_test(&bytes).unwrap();
        assert_eq!(
            store
                .tabs()
                .map(|tab| u64::from(tab.id))
                .collect::<Vec<_>>(),
            [0],
            "{location}"
        );
        assert_eq!(store.next, 2, "{location}");
    }
}

#[test]
fn a_tab_that_breaks_a_bound_refuses_the_whole_file() {
    let devices = r#"{"kind":"devices"}"#;
    let long = "x".repeat(MAX_TITLE_CHARS + 1);
    for invalid in [
        legacy(1, devices, r#","origin":"""#),
        tab(1, r#""local""#, "", devices, ""),
        tab(1, CORE_SCOPE, "w", "null", ""),
        format!(
            r#"{{"id":1,"scope":"local","workspace_id":"w","location":null,"title":"{long}"}}"#
        ),
    ] {
        let bytes = file(&[legacy(0, devices, ""), invalid], "");
        assert!(
            matches!(
                Store::restored_for_test(&bytes),
                Err(Error::InvalidBrowserTabs)
            ),
            "{}",
            String::from_utf8_lossy(&bytes)
        );
    }
    let crowd: Vec<String> = (0..=MAX_TABS as u64)
        .map(|id| legacy(id, devices, ""))
        .collect();
    assert!(matches!(
        Store::restored_for_test(&file(&crowd, "")),
        Err(Error::InvalidBrowserTabs)
    ));
    assert!(Store::restored_for_test(&file(&crowd[1..], "")).is_ok());
}
