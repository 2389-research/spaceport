#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn window_list_text_names_the_selected_core_group() {
    let group = "00000000-0000-0000-0000-000000000047";
    let mut window: WindowInfo = serde_json::from_value(serde_json::json!({
        "id":7,
        "title":"Spaceport — North",
        "active":true,
        "fullscreen":false,
        "width":1200.0,
        "height":780.0,
        "workspace":null,
        "core_workspace":{
            "hub_core_id":"hub-a",
            "machine_id":null,
            "document_name":"spaceport",
            "group_id":group,
            "current":true
        },
        "hosts":[]
    }))
    .unwrap();
    let mut text = String::new();
    describe_window(&window, &mut text);
    assert!(text.contains(&format!("core workspace {group}")), "{text}");
    assert!(text.contains("hub hub-a"), "{text}");
    assert!(text.contains("document spaceport"), "{text}");
    assert!(!text.contains("stale"), "{text}");

    window.core_workspace.as_mut().unwrap().current = false;
    let mut stale = String::new();
    describe_window(&window, &mut stale);
    assert!(stale.contains("stale"), "{stale}");
}
