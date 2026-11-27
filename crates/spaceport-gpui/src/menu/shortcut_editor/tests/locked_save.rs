use super::*;
use spaceport_keymap::Error as KeymapError;

const CHORD: &str = "ctrl-alt-shift-y";

/// A managed config and view settings file of the test's own, so no save
/// here reaches the real config directory.
struct Files {
    _temp: tempfile::TempDir,
    managed: std::path::PathBuf,
    local: std::path::PathBuf,
    view: std::path::PathBuf,
}

fn files() -> Files {
    let temp = tempfile::tempdir().unwrap();
    Files {
        managed: temp.path().join("config.toml"),
        local: temp.path().join("config.local.toml"),
        view: temp.path().join("view.toml"),
        _temp: temp,
    }
}

fn save(files: &Files, name: &str, binding: Option<Binding>) -> crate::Result<()> {
    Config::save_keybinding_at(
        &files.managed,
        &files.view,
        name,
        binding,
        crate::config::check_keymap,
    )
}

fn loaded_keymap(files: &Files) -> Keymap {
    let config = Config::load_fixture(&files.managed, &files.view).unwrap();
    crate::config::keymap(&config).unwrap()
}

/// Two editors open on the same map. The first saves `new_tab` to a chord;
/// the second, whose window still holds the older map, passes its own check
/// and saves `reload_config` to the same chord.
#[gpui::test]
fn a_stale_editor_cannot_save_a_chord_another_window_claimed(cx: &mut TestAppContext) {
    let files = files();
    save(&files, "new_tab", Some(Binding::One(CHORD.into()))).unwrap();
    let first = std::fs::read(&files.local).unwrap();

    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    let managed = files.managed.clone();
    let settings = files.view.clone();
    view.update_in(cx, |view, window, cx| {
        view.open_keybinds(window, cx);
        view.edit_shortcut(info(Command::ReloadConfig), window, cx);
        view.menu
            .shortcut_editor
            .as_ref()
            .unwrap()
            .input
            .update(cx, |input, cx| input.set_text_selected(CHORD, cx));
        view.save_shortcut_with(
            Edit::Save,
            // A save that wrongly succeeds still reports a failure, so the
            // editor never reloads from the real config directory.
            move |name, binding| {
                Config::save_keybinding_at(
                    &managed,
                    &settings,
                    name,
                    binding,
                    crate::config::check_keymap,
                )?;
                Err(std::io::Error::other("the stale save replaced the file").into())
            },
            window,
            cx,
        );
        // The window's own map has no conflict, so the save starts.
        assert!(view.menu.shortcut_editor.as_ref().unwrap().saving);
    });
    cx.run_until_parked();
    view.read_with(cx, |view, cx| {
        let editor = view.menu.shortcut_editor.as_ref().unwrap();
        assert!(!editor.saving);
        assert!(editor.error.as_ref().unwrap().contains(CHORD));
        assert_eq!(editor.input.read(cx).text(), CHORD);
        assert_eq!(view.keymap, Keymap::default());
    });
    assert_eq!(std::fs::read(&files.local).unwrap(), first);

    // The same save names the conflict as the resolver's own error.
    let result = save(&files, "reload_config", Some(Binding::One(CHORD.into())));
    assert!(
        matches!(
            result,
            Err(crate::Error::Keymap(KeymapError::DuplicateKeystroke { .. }))
        ),
        "{result:?}"
    );
    assert_eq!(std::fs::read(&files.local).unwrap(), first);
    assert_eq!(loaded_keymap(&files).primary(Command::Tab), CHORD);
}

#[::std::prelude::v1::test]
fn unbind_and_reset_save_through_the_keymap_check() {
    let files = files();
    save(&files, "new_tab", Some(Binding::One(CHORD.into()))).unwrap();
    save(&files, "new_tab", Some(Binding::Many(Vec::new()))).unwrap();
    assert_eq!(loaded_keymap(&files).primary(Command::Tab), "");
    // The freed chord is free for another command.
    save(&files, "reload_config", Some(Binding::One(CHORD.into()))).unwrap();
    save(&files, "new_tab", None).unwrap();
    let keymap = loaded_keymap(&files);
    assert_eq!(
        keymap.primary(Command::Tab),
        crate::controls::platform_default("cmd-t")
    );
    assert_eq!(keymap.primary(Command::ReloadConfig), CHORD);
}
