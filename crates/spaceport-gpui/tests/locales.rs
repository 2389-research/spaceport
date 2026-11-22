//! The build script's checks on the Fluent files, which are the only
//! definition of a valid file (docs/i18n.md).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../locales.rs"]
mod locales;

use locales::{Error, generate};
use std::{fs, path::Path};

/// Writes `files` as `locales/<language>/<feature>.ftl` and runs the build's
/// check over them.
fn check(files: &[(&str, &str)]) -> Result<String, Error> {
    let manifest = tempfile::tempdir().unwrap();
    for (path, source) in files {
        let path = manifest.path().join("locales").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    generate(manifest.path(), manifest.path())?;
    Ok(fs::read_to_string(manifest.path().join("messages.rs")).unwrap())
}

#[test]
fn a_translation_with_an_id_english_lacks_is_an_error_naming_both() {
    let error = check(&[
        ("en-US/menu.ftl", "menu-quit = Quit\n"),
        ("fr/menu.ftl", "menu-quit = Quitter\nmenu-gone = Parti\n"),
    ])
    .expect_err("fr defines menu-gone, which en-US lacks");
    assert_eq!(error.to_string(), "menu-gone in fr has no en-US message");
    assert!(matches!(
        error,
        Error::Untranslated { language, id } if language == "fr" && id == "menu-gone"
    ));
}

#[test]
fn a_translation_may_leave_english_messages_out() {
    let code = check(&[
        ("en-US/menu.ftl", "menu-quit = Quit\nmenu-open = Open\n"),
        ("fr/menu.ftl", "menu-quit = Quitter\n"),
    ])
    .unwrap();
    assert!(code.contains(r#"("menu-open", &[])"#), "{code}");
    assert!(code.contains(r#"language: "fr""#), "{code}");
}

#[test]
fn braces_and_quotes_fluent_allows_are_valid() {
    let code = check(&[(
        "en-US/menu.ftl",
        concat!(
            "# Menu text.\n",
            "menu-brace = { \"{\" } and { \"\\\"}\" }\n",
            "menu-size = { NUMBER($size, style: \"unit\") ->\n",
            "    [one] One \"{ $size }\" unit\n",
            "   *[other] { $size } units, { \"}\" } and \"more\n",
            "}\n",
            "menu-long =\n",
            "    First line\n",
            "\n",
            "    second line of { menu-brace }\n",
            "-brand = Spaceport\n",
        ),
    )])
    .unwrap();
    assert!(code.contains(r#"("menu-size", &["size"])"#), "{code}");
    assert!(!code.contains("brand"), "{code}");
}

#[test]
fn a_malformed_misprefixed_or_repeated_message_is_an_error() {
    let english = |source| check(&[("en-US/menu.ftl", source)]);
    assert!(matches!(
        english("menu-quit = Quit\nnot a message\n"),
        Err(Error::Parse(path, _)) if path.ends_with(Path::new("en-US/menu.ftl"))
    ));
    assert!(matches!(
        english("menu-quit = { $name\n"),
        Err(Error::Parse(..))
    ));
    assert!(matches!(
        english("menu-quit = Quit\nquit = Quit\n"),
        Err(Error::Prefix { id, .. }) if id == "quit"
    ));
    assert!(matches!(
        english("menu-quit = Quit\nmenu-quit = Again\n"),
        Err(Error::Duplicate { language, id }) if language == "en-US" && id == "menu-quit"
    ));
    assert!(matches!(
        check(&[("fr/menu.ftl", "menu-quit = Quitter\n")]),
        Err(Error::Missing(_))
    ));
}
