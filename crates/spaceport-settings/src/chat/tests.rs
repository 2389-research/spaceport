#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn reading_width_defaults_to_800_and_accepts_full_or_bounded_columns() {
    use crate::Config;
    assert_eq!(ChatConfig::default().max_width, DEFAULT_MAX_WIDTH);
    assert_eq!(Config::parse("").unwrap().chat.max_width, DEFAULT_MAX_WIDTH);
    assert_eq!(
        Config::parse("[chat]\nglide = true")
            .unwrap()
            .chat
            .max_width,
        DEFAULT_MAX_WIDTH
    );
    for width in [
        0,
        *MAX_WIDTH_RANGE.start(),
        800,
        1_200,
        *MAX_WIDTH_RANGE.end(),
    ] {
        let config = Config::parse(&format!("[chat]\nmax_width = {width}\n")).unwrap();
        assert_eq!(config.chat.max_width, width);
        assert!(config.unknown_keys.is_empty());
    }
    for value in [
        "-1", "1", "319", "4001", "65535", "65536", "800.5", "nan", "'wide'",
    ] {
        assert!(
            Config::parse(&format!("[chat]\nmax_width = {value}\n")).is_err(),
            "{value}"
        );
    }
}

#[test]
fn width_preferences_preserve_other_chat_settings_and_refuse_invalid_saves() {
    use crate::{Config, preferences::Preference};
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("local.toml");
    std::fs::write(
        &path,
        "# keep\n[chat]\nlayout = 'imessage'\nglide = true\n[chat.env]\nGH_HOST = 'ghe.example'\n",
    )
    .unwrap();
    for width in [1_200, 0, DEFAULT_MAX_WIDTH] {
        Config::save_preference_path(Preference::ChatMaxWidth(width), &path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let config = Config::parse(&text).unwrap();
        assert_eq!(config.chat.max_width, width);
        assert_eq!(config.chat.layout, ChatLayoutMode::IMessage);
        assert!(config.chat.glide);
        assert_eq!(config.chat.env.get("GH_HOST").unwrap(), "ghe.example");
        assert!(text.starts_with("# keep"));
    }
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(Config::save_preference_path(Preference::ChatMaxWidth(1), &path).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), saved);
}

#[test]
fn cli_width_edits_validate_before_writing_and_unset_restores_default() {
    use crate::{Config, parse_value};
    let directory = tempfile::tempdir().unwrap();
    let local = directory.path().join("local.toml");
    let daemon = directory.path().join("daemon.toml");
    std::fs::write(&local, "# keep\n[chat]\nlayout = 'cards'\n").unwrap();
    for value in ["1200", "0"] {
        assert!(
            Config::edit_key_at(&local, &daemon, "chat.max_width", Some(parse_value(value)))
                .unwrap()
        );
        let config = Config::parse(&std::fs::read_to_string(&local).unwrap()).unwrap();
        assert_eq!(config.chat.max_width.to_string(), value);
        assert_eq!(config.chat.layout, ChatLayoutMode::Cards);
    }
    let saved = std::fs::read_to_string(&local).unwrap();
    assert!(
        Config::edit_key_at(&local, &daemon, "chat.max_width", Some(parse_value("319"))).is_err()
    );
    assert_eq!(std::fs::read_to_string(&local).unwrap(), saved);
    assert!(Config::edit_key_at(&local, &daemon, "chat.max_width", None).unwrap());
    assert_eq!(
        Config::parse(&std::fs::read_to_string(local).unwrap())
            .unwrap()
            .chat
            .max_width,
        DEFAULT_MAX_WIDTH
    );
}

#[test]
fn rendering_existing_agents_is_explicitly_opted_in() {
    assert!(!crate::Config::parse("").unwrap().chat.render_existing);
    assert!(
        crate::Config::parse("[chat]\nrender_existing = true\n")
            .unwrap()
            .chat
            .render_existing
    );
}

#[test]
fn mirrors_accept_only_bounded_explicit_endpoint_ids() {
    let config = crate::Config::parse(
        "[[chat.mirrors]]\npane = 'w1:p1'\nendpoint = '0123456789abcdef0123456789abcdef'\n",
    )
    .unwrap();
    assert_eq!(config.chat.mirrors.len(), 1);
    assert_eq!(config.chat.mirrors[0].host, None);
    assert!(
        crate::Config::parse(
            "[[chat.mirrors]]\npane = 'w1:p1'\nendpoint = '/tmp/arbitrary.sock'\n"
        )
        .is_err()
    );
    let mut chat = config.chat;
    chat.mirrors = vec![chat.mirrors[0].clone(); 17];
    assert!(matches!(chat.check(), Err(crate::Error::InvalidChatMirror)));
}

#[test]
fn the_layout_is_read_and_saved_by_name() {
    let config = crate::Config::parse("[chat]\nlayout = \"cards\"\n").unwrap();
    assert_eq!(config.chat.layout, ChatLayoutMode::Cards);
    assert!(crate::Config::parse("[chat]\nlayout = \"stack\"\n").is_err());
    // A retired layout a config may still name opens in its closest one.
    assert_eq!(
        crate::Config::parse("[chat]\nlayout = \"pill\"\n")
            .unwrap()
            .chat
            .layout,
        ChatLayoutMode::Bubbles
    );
    assert_eq!(
        crate::Config::parse("").unwrap().chat.layout,
        ChatLayoutMode::Timeline
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config-gpui.local.toml");
    std::fs::write(&path, "[chat.env]\nGH_HOST = \"ghe.example\"\n").unwrap();
    for &mode in ChatLayoutMode::ALL {
        crate::Config::save_preference_path(
            crate::preferences::Preference::ChatLayout(mode),
            &path,
        )
        .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let config = crate::Config::parse(&text).unwrap();
        assert_eq!(config.chat.layout, mode);
        assert_eq!(config.chat.env.get("GH_HOST").unwrap(), "ghe.example");
    }
}

#[test]
fn chats_jump_to_new_output_unless_glide_is_on() {
    assert!(!crate::Config::parse("").unwrap().chat.glide);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config-gpui.local.toml");
    std::fs::write(&path, "[chat]\nlayout = \"imessage\"\n").unwrap();
    for glide in [true, false] {
        crate::Config::save_preference_path(
            crate::preferences::Preference::ChatGlide(glide),
            &path,
        )
        .unwrap();
        let config = crate::Config::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config.chat.glide, glide);
        assert_eq!(config.chat.layout, ChatLayoutMode::IMessage);
    }
}

mod launcher;
