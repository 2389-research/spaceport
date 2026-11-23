use super::*;
use spaceport_settings::Config;

#[test]
fn a_visuals_identity_is_a_hash_of_what_it_draws() {
    assert_eq!(
        VisualId::of(("code", "rs", "let x = 1;")),
        VisualId::of(("code", "rs", "let x = 1;"))
    );
    // The language is part of what a code block draws, since it picks the
    // tokens' colors.
    assert_ne!(
        VisualId::of(("code", "rs", "let x = 1;")),
        VisualId::of(("code", "py", "let x = 1;"))
    );
    // The same text as a formula is another visual, so the role it is keyed
    // with keeps the two apart.
    assert_ne!(
        VisualId::of(("math", MathMode::Inline, "x")),
        VisualId::of(("code", MathMode::Inline, "x"))
    );
}

#[test]
fn code_and_math_count_against_the_rich_budget_and_pictures_do_not() {
    let Some(image) = Image::new("image/png", "AAAA".to_owned()) else {
        panic!("a bounded png");
    };
    let Ok(url) = WebUrl::try_from("https://example.com/a.png") else {
        panic!("a bound address");
    };
    let code = Source::Code {
        language: "rs".into(),
        source: "let x = 1;".into(),
    };
    let math = Source::Math {
        source: "x".into(),
        mode: MathMode::Inline,
    };
    assert_eq!(code.kind(), Kind::Code);
    assert_eq!(math.kind(), Kind::Math);
    for other in [
        Source::Image(image),
        Source::Diagram("A-->B".into()),
        Source::Remote(url),
    ] {
        assert_eq!(other.kind(), Kind::Other);
    }
}

#[test]
fn a_diagram_style_takes_its_colors_and_font_from_the_theme() {
    let theme = Theme {
        active: 0x30_30_30,
        background: 0xf0_f0_f0,
        foreground: 0x10_10_10,
        muted: 0x40_40_40,
        surface: 0x20_20_20,
        ..Theme::default()
    };
    let font = Config::default().ui;
    let style = DiagramStyle::of(&theme, &font);
    assert_eq!(style.background, theme.background);
    assert_eq!(style.border, theme.muted);
    assert_eq!(style.fill, theme.surface);
    assert_eq!(style.line, theme.subtext());
    assert_eq!(style.note, theme.active);
    assert_eq!(style.text, theme.foreground);
    assert_eq!(style.family, font.family);
    assert_eq!(style.size, font.size);
}

#[test]
fn a_dark_background_asks_for_the_dark_diagram() {
    let style = |background| {
        DiagramStyle::of(
            &Theme {
                background,
                ..Theme::default()
            },
            &Config::default().ui,
        )
    };
    assert!(style(0x10_10_10).dark);
    assert!(!style(0xff_ff_ff).dark);
    // Green carries most of the luminance and blue the least, so a saturated
    // background is measured rather than read off one channel.
    assert!(luminance(0x00_00_ff) < 0.5);
    assert!(luminance(0x00_ff_00) > 0.5);
}
