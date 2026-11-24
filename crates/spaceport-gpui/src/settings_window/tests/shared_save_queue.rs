// ABOUTME: Verifies queued shared edits against isolated real configuration files.
// ABOUTME: Preserves external conflict refusal while draining owned writes.
use super::*;
use crate::herdr_settings::Edit;
use std::path::{Path, PathBuf};

fn loaded(path: &Path) -> crate::Result<Loaded> {
    let mut loaded = fixture();
    loaded.shared = Some(herdr_settings::Settings::load_fixture(path.to_owned())?);
    Ok(loaded)
}

fn load_file(path: PathBuf) -> impl FnOnce() -> crate::Result<Loaded> + Send {
    move || loaded(&path)
}

fn start_file(temp: &tempfile::TempDir) -> PathBuf {
    let path = temp.path().join("config.toml");
    std::fs::write(
        &path,
        "[ui]\ncopy_on_select = false\n[ui.sound]\nenabled = true\n",
    )
    .unwrap();
    path
}

#[gpui::test]
fn accepted_shared_edits_drain_to_disk_before_close_or_quit(cx: &mut TestAppContext) {
    for (close, quit) in [(false, false), (true, false), (false, true)] {
        let temp = tempfile::tempdir().unwrap();
        let path = start_file(&temp);
        let (view, cx) = cx.add_window_view(skill_fixture);
        let task = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.shared = Some(herdr_settings::Settings::load_fixture(path.clone()).unwrap());
                view.save_shared_with(Edit::Sound(false), load_file(path.clone()), cx);
                view.save_shared_with(Edit::CopyOnSelect(true), load_file(path.clone()), cx);
                view.save_shared_with(Edit::HideSingleTabBar(true), load_file(path.clone()), cx);
                if quit {
                    Some(view.shutdown_with(|_| panic!("no sizes"), cx))
                } else {
                    if close {
                        assert!(!view.should_close(window, cx));
                    }
                    None
                }
            })
        });
        let (done, result) = std::sync::mpsc::sync_channel(1);
        if let Some(task) = task {
            cx.executor()
                .spawn(async move {
                    done.send(task.await).unwrap();
                })
                .detach();
        }
        cx.run_until_parked();
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(10));
        cx.run_until_parked();
        if quit {
            result.try_recv().unwrap().unwrap();
        }
        let saved = herdr_settings::Settings::load_fixture(path.clone()).unwrap();
        assert!(!saved.sound_enabled);
        assert!(
            saved.copy_on_select,
            "the second accepted edit must reach disk"
        );
        assert!(
            saved.hide_tab_bar_when_single_tab,
            "later edits must follow the whole owned write chain"
        );
    }
}

#[gpui::test]
fn external_edit_between_owned_write_and_reload_is_not_adopted_for_the_queue(
    cx: &mut TestAppContext,
) {
    for (quit, after_read) in [(false, false), (true, false), (false, true), (true, true)] {
        let temp = tempfile::tempdir().unwrap();
        let path = start_file(&temp);
        let (view, cx) = cx.add_window_view(skill_fixture);
        let task = view.update(cx, |view, cx| {
            view.shared = Some(herdr_settings::Settings::load_fixture(path.clone()).unwrap());
            let external = path.clone();
            view.save_shared_with(
                Edit::Sound(false),
                move || {
                    // Cover replacement both before and after the reconciliation read.
                    let before_external = after_read.then(|| loaded(&external));
                    std::fs::write(&external, "# external\n[ui.sound]\nenabled = true\n")?;
                    match before_external {
                        Some(loaded) => loaded,
                        None => loaded(&external),
                    }
                },
                cx,
            );
            view.save_shared_with(Edit::CopyOnSelect(true), load_file(path.clone()), cx);
            quit.then(|| view.shutdown_with(|_| panic!("no sizes"), cx))
        });
        let (done, result) = std::sync::mpsc::sync_channel(1);
        if let Some(task) = task {
            cx.executor()
                .spawn(async move {
                    done.send(task.await).unwrap();
                })
                .detach();
        }
        cx.run_until_parked();
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(10));
        cx.run_until_parked();
        if quit {
            assert!(result.try_recv().unwrap().is_err());
        } else {
            view.read_with(cx, |view, _| assert!(view.error.is_some()));
        }
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "# external\n[ui.sound]\nenabled = true\n"
        );
    }
}

#[gpui::test]
fn shared_edit_queued_behind_a_captured_theme_write_uses_its_owned_snapshot(
    cx: &mut TestAppContext,
) {
    let temp = tempfile::tempdir().unwrap();
    let path = start_file(&temp);
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        let shared = herdr_settings::Settings::load_fixture(path.clone()).unwrap();
        view.shared = Some(shared.clone());
        view.save_kind_with_snapshot(
            move || {
                persistence::SharedWrite::save(shared, Edit::Theme("nord".into()))
                    .map(Some)
                    .map_err(crate::Error::from)
            },
            load_file(path.clone()),
            true,
            Kind::Theme(0),
            cx,
        );
        view.save_shared_with(Edit::CopyOnSelect(true), load_file(path.clone()), cx);
    });
    cx.run_until_parked();
    let saved = herdr_settings::Settings::load_fixture(path).unwrap();
    assert_eq!(saved.theme_name, "nord");
    assert!(saved.copy_on_select);
    view.read_with(cx, |view, _| assert!(view.error.is_none()));
}
