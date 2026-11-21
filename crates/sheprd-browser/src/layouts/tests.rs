#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn core_layouts_keep_document_machine_and_page_pick_separate() {
    let group = uuid::Uuid::from_u128(77);
    let layout = r#"{"groups":[{"pick":{"kind":"page","id":41},"share":1}],"active":0}"#;
    let file = format!(
        r#"{{"layouts":[
        {{"scope":"local:/x.sock","workspace":"{group}","layout":{layout}}},
        {{"scope":{{"kind":"core","hub_core_id":"hub","machine_id":"machine-a","document_name":"spaceport"}},"workspace":"{group}","layout":{layout}}},
        {{"scope":{{"kind":"core","hub_core_id":"hub","machine_id":"machine-b","document_name":"spaceport"}},"workspace":"{group}","layout":{layout}}},
        {{"scope":{{"kind":"core","hub_core_id":"hub","machine_id":"machine-a","document_name":"other"}},"workspace":"{group}","layout":{layout}}}
    ]}}"#
    );
    let parsed = parse(file.as_bytes()).unwrap();
    assert_eq!(parsed.len(), 4);
    let expected: SavedLayout = serde_json::from_str(layout).unwrap();
    for (machine, document) in [
        ("machine-a", "spaceport"),
        ("machine-b", "spaceport"),
        ("machine-a", "other"),
    ] {
        let key = super::super::CoreGroupKey::new("hub", Some(machine), document, group)
            .unwrap()
            .parts();
        assert_eq!(parsed.get(&key), Some(&expected));
    }
    let old = (Scope::local("/x.sock".as_ref()), group.to_string());
    assert_eq!(parsed.get(&old), Some(&expected));
    let invalid = format!(
        r#"{{"layouts":[{{"scope":{{"kind":"core","hub_core_id":"hub","machine_id":null,"document_name":"spaceport"}},"workspace":"{group}","layout":{{"groups":[{{"pick":{{"kind":"herdr","id":"t1"}},"share":1}}],"active":0}}}}]}}"#
    );
    assert!(parse(invalid.as_bytes()).is_err());
    let alternate = format!(
        r#"{{"layouts":[{{"scope":{{"kind":"core","hub_core_id":"hub","machine_id":null,"document_name":"spaceport"}},"workspace":"{}","layout":{layout}}}]}}"#,
        group.to_string().to_uppercase()
    );
    assert!(parse(alternate.as_bytes()).is_err());
}

#[test]
fn saved_layouts_round_trip_and_damaged_files_are_refused() {
    let layout = r#"{"groups":[{"pick":null,"share":0.5},{"pick":{"kind":"page","id":3},"share":0.5}],"active":1}"#;
    let file = format!(
        r#"{{"layouts":[{{"scope":"local:/x.sock","workspace":"w1","layout":{layout}}}]}}"#
    );
    let parsed = parse(file.as_bytes()).unwrap();
    let key = (Scope::local("/x.sock".as_ref()), "w1".to_owned());
    let saved = parsed.get(&key).unwrap();
    assert_eq!(serde_json::to_string(saved).unwrap(), layout);
    let bytes = serde_json::to_vec(&Saved {
        layouts: vec![Entry {
            scope: key.0.clone(),
            workspace: key.1.clone(),
            layout: saved.clone(),
        }],
    })
    .unwrap();
    assert_eq!(parse(&bytes).unwrap(), parsed);
    for invalid in [
        r#"{"layouts":[{"scope":"s","workspace":"","layout":{"groups":[{"pick":null,"share":1}],"active":0}}]}"#,
        r#"{"layouts":[{"scope":"s","workspace":"w","layout":{"groups":[],"active":0}}]}"#,
        r#"{"layouts":[{"scope":"s","workspace":"w","layout":{"groups":[{"pick":{"kind":"file","id":"x"},"share":1}],"active":0}}]}"#,
        r#"{"layouts":"#,
    ] {
        assert!(parse(invalid.as_bytes()).is_err(), "{invalid}");
    }
}

#[gpui::test]
fn records_write_changes_only_and_closed_workspaces_are_forgotten(cx: &mut gpui::TestAppContext) {
    let layout: SavedLayout =
        serde_json::from_str(r#"{"groups":[{"pick":null,"share":1}],"active":0}"#).unwrap();
    let scope = Scope::endpoint("local");
    let (one, two) = (
        (scope.clone(), "w1".to_owned()),
        (Scope::endpoint("ssh:box"), "w1".to_owned()),
    );
    cx.update(|cx| {
        // Forgetting what was never saved claims nothing.
        Layouts::record(cx, &one, None);
        assert!(!cx.has_global::<Layouts>());
        Layouts::record(cx, &one, Some(layout.clone()));
        Layouts::record(cx, &two, Some(layout.clone()));
        assert_eq!(Layouts::snapshot(cx).len(), 2);
        Layouts::forget_workspaces(cx, &scope, &["w1".to_owned()]);
        let left = Layouts::snapshot(cx);
        assert!(!left.contains_key(&one) && left.contains_key(&two));
        Layouts::record(cx, &two, None);
        assert!(Layouts::snapshot(cx).is_empty());
    });
}
