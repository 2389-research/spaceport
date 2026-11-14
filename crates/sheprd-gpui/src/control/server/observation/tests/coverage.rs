// ABOUTME: Keeps unavailable runtime capabilities separate from observed device connections.
// ABOUTME: Checks the actual broker's wire output so model input cannot invent a Core host.
use super::*;

#[gpui::test]
fn coverage_producer_distinguishes_runtime_support_from_connections(cx: &mut gpui::TestAppContext) {
    let (_, cx) = cx.add_window_view(fixture_window);
    cx.cx.update(|cx| {
        let (_, coverage) = list(cx);
        let encoded = serde_json::to_value(coverage).unwrap();
        let rows = encoded.as_array().unwrap();
        let runtime = &rows[0];
        assert_eq!(runtime["scope"], "runtime");
        assert_eq!(runtime["name"], "Herdr Core");
        assert_eq!(runtime["unavailable"], "disabled");
        assert!(runtime.get("host").is_none());
        assert!(
            rows.iter()
                .any(|row| row["scope"] == "connection" && row["host"].is_string())
        );
    });
}
