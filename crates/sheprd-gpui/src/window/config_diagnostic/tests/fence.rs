// ABOUTME: Pins which connection changes a config diagnostic card's action survives.
// ABOUTME: The card reaches the connection it was drawn from while that connection exists.

use super::{super::DrawnFrom, *};
use crate::endpoint::fence::changes::{CLICK_CHANGES, CLICK_REACHES, apply_click};

#[gpui::test]
fn a_card_reaches_the_connection_it_was_drawn_from_while_it_exists(cx: &mut TestAppContext) {
    let seen = CLICK_CHANGES.map(|change| {
        let (view, cx) = cx.add_window_view(fixture_window);
        view.update(cx, |view, _| {
            view.endpoints.push(crate::endpoint::Endpoint::new(
                "remote".into(),
                "Remote".into(),
                sheprd_client::ConnectTarget::Socket("/unused-config-diagnostic.sock".into()),
                true,
            ));
        });
        set_diagnostic(&view, 0, Some("config.toml has unknown keys"), cx);
        view.update(cx, |view, cx| {
            view.live = view.endpoints[0].live.clone();
            let endpoint = &view.endpoints[0];
            let (id, generation, inbox, lines) = (
                endpoint.id.clone(),
                endpoint.generation,
                endpoint.connection.inbox.clone(),
                endpoint.config_diagnostic.visible().unwrap().clone(),
            );
            apply_click(view, 0, change, cx);
            let drawn = DrawnFrom {
                connection: ConnectionRef::for_test(&id, generation, &inbox),
                lines,
            };
            view.dismiss_config_diagnostic(&drawn, cx);
            view.endpoints[0].config_diagnostic.visible().is_none()
        })
    });
    assert_eq!(seen, CLICK_REACHES);
}
