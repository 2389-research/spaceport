use super::{
    super::super::{
        agents::Indicators,
        layout,
        paint::PLAIN,
        row::RowContext,
        tokens::{ResolvedToken, TextRole, TokenKind},
    },
    dot,
};
use spaceport_client::protocol::AgentStatus;
use spaceport_settings::{FontConfig, LayoutMode, StatusIcons, Theme, TokenStyle, Weight};

#[core::prelude::v1::test]
fn the_slot_dot_follows_a_configured_leading_state_icon() {
    let font = FontConfig {
        family: "Menlo".into(),
        size: 12.,
        fallbacks: None,
        line_height_multiple: None,
        weight: Weight::NORMAL,
    };
    let theme = Theme::default();
    let indicators = Indicators::new(None, StatusIcons::default(), false, false, &theme);
    let cx = RowContext {
        indicators,
        font: &font,
        theme: &theme,
        look: layout::for_mode(LayoutMode::Superset),
        width: 232.,
        nest: 0.,
        mark: Default::default(),
        host: None,
        right_to_left: false,
        painters: PLAIN,
    };
    let working = AgentStatus::Working;
    let native = indicators.color(working);
    let icon = |style| ResolvedToken {
        kind: TokenKind::StateIcon,
        style,
    };
    let name = ResolvedToken {
        kind: TokenKind::Text("repo".into(), TextRole::Workspace),
        style: TokenStyle::default(),
    };
    let red = TokenStyle {
        fg: Some(0xff0000),
        bold: Some(true),
        dim: None,
    };
    assert_eq!(dot(&[], working, &cx), (working, Some((native, false))));
    assert_eq!(
        dot(
            &[vec![icon(TokenStyle::default()), name.clone()]],
            working,
            &cx
        ),
        (working, Some((native, false)))
    );
    assert_eq!(
        dot(&[vec![icon(red), name.clone()]], working, &cx),
        (working, Some((0xff0000, true)))
    );
    // No leading status, or nothing to report, draws no dot.
    assert_eq!(dot(&[vec![name.clone()]], working, &cx), (working, None));
    assert_eq!(
        dot(&[vec![icon(red), name]], AgentStatus::Unknown, &cx),
        (AgentStatus::Unknown, None)
    );
}
