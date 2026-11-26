// ABOUTME: Checks that GPUI can render the multicolor artwork embedded for sidebar rows.
// ABOUTME: Looks for the Gemini mark's different hues in the decoded pixels.
use super::*;

#[gpui::test]
fn sidebar_gemini_art_renders_multiple_brand_hues(cx: &mut TestAppContext) {
    let bytes = Icons.load("icons/brand/gemini.svg").unwrap().unwrap();
    let image = Image::from_bytes(ImageFormat::Svg, bytes.into_owned())
        .to_image_data(cx.update(|cx| cx.svg_renderer()))
        .unwrap();
    let pixels = image.as_bytes(0).unwrap().as_chunks::<4>().0;
    // GPUI stores BGRA. The logo has visibly blue and red regions.
    assert!(
        pixels
            .iter()
            .any(|&[b, g, r, a]| a > 240 && b > 180 && r < 100 && b > g)
    );
    assert!(
        pixels
            .iter()
            .any(|&[b, g, r, a]| a > 240 && r > 180 && b < 120 && r > g)
    );
}

#[gpui::test]
fn sidebar_paints_brand_colors_and_theme_ink_in_both_appearances(cx: &mut TestAppContext) {
    use gpui::Styled;
    use spaceport_settings::Theme;
    cx.update(|_| {
        for name in ["Default", "Catppuccin Latte"] {
            let theme = Theme::builtin(name).unwrap();
            for neutral in [theme.muted, theme.foreground] {
                for (agent, expected) in [
                    (AgentIcon::Claude, theme.ink(0xd97757)),
                    (AgentIcon::Pi, neutral),
                    (AgentIcon::Generic, neutral),
                ] {
                    let mut element = agent.sidebar_element(16., neutral, &theme);
                    let glyph = element.downcast_mut::<gpui::Svg>().unwrap();
                    assert_eq!(glyph.text_style().color, Some(gpui::rgb(expected).into()));
                }
                for agent in [AgentIcon::Codex, AgentIcon::Gemini, AgentIcon::Omo] {
                    let mut element = agent.sidebar_element(16., neutral, &theme);
                    assert!(element.downcast_mut::<gpui::Img>().is_some());
                }
                for agent in [AgentIcon::Kimi, AgentIcon::Qodercli] {
                    let mut element = agent.sidebar_element(16., neutral, &theme);
                    assert!(element.downcast_mut::<gpui::Div>().is_some());
                }
            }
        }
    });
}
