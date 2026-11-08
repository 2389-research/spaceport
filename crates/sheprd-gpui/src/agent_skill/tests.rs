#![allow(clippy::unwrap_used)]
use super::*;

mod branding;

#[test]
fn the_skill_is_marked_and_names_this_executable() {
    assert!(SKILL.starts_with("---\nname: herdr-gpui-browser\n"));
    assert!(managed(SKILL));
    assert!(SKILL.contains("local `sheprd browser open` command has been removed"));
    assert_eq!(text(None), SKILL);
    let bundled = text(Some(Path::new(
        "/Applications/Herdr.app/Contents/MacOS/Herdr",
    )));
    assert!(!bundled.contains("herdr-gpui browser"));
    // Every command, not only the browser's, names the executable.
    assert!(!bundled.contains("herdr-gpui "));
    assert!(bundled.contains("/Applications/Herdr.app/Contents/MacOS/Herdr status"));
    let spaced = text(Some(Path::new("/Users/a b/it's/Herdr")));
    assert!(spaced.contains(r"'/Users/a b/it'\''s/Herdr' status"));
}

/// Agent Skills reads the description as YAML. Keep it a quoted scalar so
/// punctuation cannot change how agents parse the skill frontmatter.
#[test]
fn the_skill_description_is_a_quoted_scalar() {
    assert!(!description(SKILL).contains('"'));
}

#[test]
fn installing_the_browser_skill_does_not_touch_an_existing_automation_skill() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join(".claude");
    let legacy = file(&root, "herdr-automations");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(&legacy, "my automation notes").unwrap();
    install(home.path(), SKILL).unwrap();
    assert_eq!(
        std::fs::read_to_string(&legacy).unwrap(),
        "my automation notes"
    );
    remove(home.path()).unwrap();
    assert_eq!(
        std::fs::read_to_string(&legacy).unwrap(),
        "my automation notes"
    );
}

#[test]
fn browser_skill_installs_and_removes() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".claude")).unwrap();
    let placed = install(home.path(), SKILL).unwrap();
    assert_eq!(placed.len(), 1);
    let removed = remove(home.path()).unwrap();
    assert_eq!(removed, placed);
}

/// The Agent Skills format caps a description at 1,024 characters, and
/// agents such as Pi refuse a skill over it.
#[test]
fn the_skill_description_fits_the_agent_skills_limit() {
    for skill in [SKILL] {
        let length = description(skill).chars().count();
        assert!((1..=1024).contains(&length), "{length} characters");
    }
}

/// The frontmatter's description, without its quotes.
fn description(skill: &str) -> &str {
    skill
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .and_then(|(frontmatter, _)| {
            frontmatter
                .lines()
                .find_map(|line| line.strip_prefix("description: "))
        })
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .unwrap()
}

#[test]
fn the_skill_does_not_promise_local_page_open() {
    assert!(SKILL.contains("The local `sheprd browser open` command has been removed"));
    assert!(!SKILL.contains("SSH host with the installed shim"));
    assert!(!SKILL.contains("sheprd browser open ./"));
    assert!(!SKILL.contains("HerdrKit.scenario("));
}

#[test]
fn the_skill_does_not_promise_code_open() {
    assert!(!SKILL.contains("sheprd code open"));
}

#[test]
fn installing_keeps_to_existing_agent_directories_and_the_users_own_files() {
    let home = tempfile::tempdir().unwrap();
    assert!(matches!(
        install(home.path(), SKILL),
        Err(crate::Error::SkillInstall { .. })
    ));
    assert!(!agents_present(home.path()));
    std::fs::create_dir(home.path().join(".agents")).unwrap();
    std::fs::create_dir(home.path().join(".claude")).unwrap();
    // A skill of the same name the user wrote is theirs.
    let own = file(&home.path().join(".claude"), NAME);
    std::fs::create_dir_all(own.parent().unwrap()).unwrap();
    std::fs::write(&own, "my own skill").unwrap();
    let placed = install(home.path(), SKILL).unwrap();
    assert_eq!(placed, [file(&home.path().join(".agents"), NAME)]);
    assert_eq!(std::fs::read_to_string(&placed[0]).unwrap(), SKILL);
    assert_eq!(std::fs::read_to_string(&own).unwrap(), "my own skill");

    // A newer text replaces the managed copy; the same text is a no-op.
    let newer = format!("{SKILL}\nNew section\n");
    install(home.path(), &newer).unwrap();
    assert_eq!(std::fs::read_to_string(&placed[0]).unwrap(), newer);

    // Removing takes only managed copies and their now-empty folders.
    let removed = remove(home.path()).unwrap();
    let browser = removed
        .iter()
        .filter(|path| path.parent().is_some_and(|dir| dir.ends_with(NAME)))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(browser, placed);
    assert!(removed.iter().all(|path| path != &own));
    assert!(!placed[0].parent().unwrap().exists());
    assert_eq!(std::fs::read_to_string(&own).unwrap(), "my own skill");
    assert!(remove(home.path()).unwrap().is_empty());
}

#[test]
fn choices_are_saved_as_json() {
    let text = serde_json::to_string(&Saved {
        choice: Choice::Declined,
    })
    .unwrap();
    assert_eq!(text, r#"{"choice":"declined"}"#);
    let saved: Saved = serde_json::from_str(r#"{"choice":"installed"}"#).unwrap();
    assert_eq!(saved.choice, Choice::Installed);
}

mod install_setting;

#[gpui::test]
fn the_offer_is_made_once_and_closing_it_means_not_now(cx: &mut gpui::TestAppContext) {
    use crate::sidebar::layout_tests::fixture_window;
    cx.update(|cx| cx.set_global(AgentSkill::unasked()));
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.active = true;
            view.offer_browser_skill(window, cx);
            assert_eq!(view.menu.page, Some(crate::menu::Page::AgentSkill));
            // Another window, or a later tick, does not ask again.
            view.dismiss_menu(window, cx);
            view.offer_browser_skill(window, cx);
            assert_eq!(view.menu.page, None);
        });
        assert_eq!(AgentSkill::choice(cx), Some(Choice::Declined));
    });
}
