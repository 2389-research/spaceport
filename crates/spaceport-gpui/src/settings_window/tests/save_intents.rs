// ABOUTME: Protects choices accepted while Settings persistence is running.
// ABOUTME: Exercises serial flushes and shutdown against captured write intents.
use super::*;
use crate::config::LayoutMode;

#[gpui::test]
fn flush_keeps_a_newer_layout_chosen_during_its_write(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(Mutex::new(Vec::new()));
    view.update(cx, |view, cx| {
        view.layout_io = Some(recording_layouts(writes.clone(), false));
        view.accept_layout_choice(LayoutMode::Orca, cx);
        view.save_pending_drafts(cx);
        assert!(view.busy());
        view.accept_layout_choice(LayoutMode::Minimal, cx);
    });
    cx.run_until_parked();
    assert_eq!(
        *writes.lock().unwrap(),
        [LayoutMode::Orca, LayoutMode::Minimal]
    );
    view.read_with(cx, |view, _| {
        assert_eq!(view.config.layout.mode, LayoutMode::Minimal);
        assert!(view.layout_intent.is_none());
        assert!(!view.busy());
    });
}

#[gpui::test]
fn flush_keeps_a_newer_theme_chosen_during_its_write(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(Mutex::new(Vec::new()));
    view.update(cx, |view, cx| {
        view.theme_io = Some(recording_themes(writes.clone(), false));
        choose(view, "Nord", cx);
        view.save_pending_drafts(cx);
        assert!(view.busy());
        choose(view, "Dracula", cx);
    });
    cx.run_until_parked();
    assert_eq!(*writes.lock().unwrap(), ["Nord", "Dracula"]);
    view.read_with(cx, |view, _| {
        assert_eq!(view.config.theme, "Dracula");
        assert!(!view.theme_dirty());
        assert!(!view.busy());
    });
}

#[gpui::test]
fn quit_writes_choices_newer_than_the_running_theme_and_layout(cx: &mut TestAppContext) {
    for theme in [false, true] {
        let (view, cx) = cx.add_window_view(skill_fixture);
        let themes = Arc::new(Mutex::new(Vec::new()));
        let layouts = Arc::new(Mutex::new(Vec::new()));
        let quit = view.update(cx, |view, cx| {
            view.theme_io = Some(recording_themes(themes.clone(), false));
            view.layout_io = Some(recording_layouts(layouts.clone(), false));
            if theme {
                choose(view, "Nord", cx);
                view.save_pending_drafts(cx);
                choose(view, "Dracula", cx);
            } else {
                view.accept_layout_choice(LayoutMode::Orca, cx);
                view.save_pending_drafts(cx);
                view.accept_layout_choice(LayoutMode::Minimal, cx);
            }
            view.shutdown_with(|_| panic!("no font edits"), cx)
        });
        let (done, result) = std::sync::mpsc::sync_channel(1);
        cx.executor()
            .spawn(async move {
                done.send(quit.await).unwrap();
            })
            .detach();
        cx.run_until_parked();
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(10));
        cx.run_until_parked();
        result.try_recv().unwrap().unwrap();
        if theme {
            assert_eq!(*themes.lock().unwrap(), ["Nord", "Dracula"]);
        } else {
            assert_eq!(
                *layouts.lock().unwrap(),
                [LayoutMode::Orca, LayoutMode::Minimal]
            );
        }
    }
}

#[gpui::test]
fn ordinary_edits_accepted_during_a_save_are_drained_on_close_and_quit(cx: &mut TestAppContext) {
    use crate::config::preferences::Preference;
    for quit in [false, true] {
        let (view, cx) = cx.add_window_view(skill_fixture);
        let writes = Arc::new(Mutex::new(Vec::new()));
        let task = cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                let captured = writes.clone();
                view.controls.preference_io = Some(controls::preferences::PreferenceIo::recording(
                    move |edit| {
                        captured.lock().unwrap().push(edit);
                    },
                ));
                view.save_preference(Preference::SmoothScroll(false), cx);
                view.save_preference(Preference::ConfirmCloseTab(false), cx);
                if quit {
                    Some(view.shutdown_with(|_| panic!("no sizes"), cx))
                } else {
                    assert!(!view.should_close(window, cx));
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
        assert_eq!(
            *writes.lock().unwrap(),
            [
                Preference::SmoothScroll(false),
                Preference::ConfirmCloseTab(false)
            ]
        );
    }
}

#[gpui::test]
fn pending_writes_are_bounded_and_saturation_reports_the_refusal(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(AtomicUsize::new(0));
    view.update(cx, |view, cx| {
        view.save_with(|| Ok(()), fixture_load, false, cx);
        for _ in 0..spaceport_settings::save_queue::QUEUE_LIMIT {
            let writes = writes.clone();
            view.save_with(
                move || {
                    writes.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
                fixture_load,
                false,
                cx,
            );
        }
        view.save_with(
            || panic!("full queue rejects this write"),
            fixture_load,
            false,
            cx,
        );
        assert!(
            view.error
                .as_ref()
                .unwrap()
                .contains("Too many pending changes")
        );
    });
    cx.run_until_parked();
    assert_eq!(
        writes.load(Ordering::SeqCst),
        spaceport_settings::save_queue::QUEUE_LIMIT
    );
}

#[gpui::test]
fn failed_write_keeps_newer_theme_and_layout_for_explicit_retry(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(Mutex::new(Vec::new()));
    view.update(cx, |view, cx| {
        view.theme_io = Some(recording_themes(writes.clone(), true));
        choose(view, "Nord", cx);
        view.save_pending_drafts(cx);
        choose(view, "Dracula", cx);
        view.accept_layout_choice(LayoutMode::Orca, cx);
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.config.theme, "Dracula");
        assert_eq!(view.layout_intent, Some(LayoutMode::Orca));
        assert!(view.theme_dirty());
        assert!(!view.persistence.flushing());
        assert!(view.error.is_some());
    });
    assert_eq!(*writes.lock().unwrap(), ["Nord"]);
}

#[gpui::test]
fn later_queued_success_does_not_hide_a_failed_accepted_edit(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        view.save_with(|| Err(crate::Error::MissingHome), fixture_load, false, cx);
        view.save_with(|| Ok(()), fixture_load, false, cx);
    });
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(!view.busy());
        assert!(
            view.error.is_some(),
            "the failed edit still needs attention"
        );
    });
}

#[gpui::test]
fn ordinary_edit_accepted_behind_a_reload_survives_close(cx: &mut TestAppContext) {
    use crate::config::preferences::Preference;
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(Mutex::new(Vec::new()));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let captured = writes.clone();
            view.controls.preference_io = Some(controls::preferences::PreferenceIo::recording(
                move |edit| {
                    captured.lock().unwrap().push(edit);
                },
            ));
            view.reload_with(fixture_load, cx);
            view.save_preference(Preference::SmoothScroll(false), cx);
            assert!(!view.should_close(window, cx));
        })
    });
    cx.run_until_parked();
    assert_eq!(*writes.lock().unwrap(), [Preference::SmoothScroll(false)]);
}

#[gpui::test]
fn quit_reports_a_queued_failure_and_still_drains_later_edits(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(AtomicUsize::new(0));
    let quit = view.update(cx, |view, cx| {
        view.save_with(|| Ok(()), fixture_load, false, cx);
        view.save_with(|| Err(crate::Error::MissingHome), fixture_load, false, cx);
        let writes = writes.clone();
        view.save_with(
            move || {
                writes.fetch_add(1, Ordering::SeqCst);
                Ok(())
            },
            fixture_load,
            false,
            cx,
        );
        view.shutdown_with(|_| panic!("no sizes"), cx)
    });
    let (done, result) = std::sync::mpsc::sync_channel(1);
    cx.executor()
        .spawn(async move {
            done.send(quit.await).unwrap();
        })
        .detach();
    cx.run_until_parked();
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(10));
    cx.run_until_parked();
    assert!(matches!(
        result.try_recv().unwrap(),
        Err(crate::Error::MissingHome)
    ));
    assert_eq!(writes.load(Ordering::SeqCst), 1);
}

#[gpui::test]
fn a_queued_sidebar_commit_does_not_commit_a_later_preview(cx: &mut TestAppContext) {
    use crate::config::preferences::Preference;
    let (view, cx) = cx.add_window_view(skill_fixture);
    let disk = Arc::new(Mutex::new(Config::default()));
    let writes = Arc::new(Mutex::new(Vec::new()));
    view.update(cx, |view, cx| {
        let (disk, loaded, writes) = (disk.clone(), disk.clone(), writes.clone());
        view.controls.preference_io = Some(controls::preferences::PreferenceIo::with_load(
            move |edit| {
                let mut config = disk.lock().unwrap();
                match &edit {
                    Preference::SidebarIndent(value) => {
                        config.sidebar_style.overrides.indent = *value
                    }
                    Preference::SidebarRowPadding(value) => {
                        config.sidebar_style.overrides.row_padding = *value
                    }
                    other => panic!("unexpected write {other:?}"),
                }
                writes.lock().unwrap().push(edit);
            },
            move || {
                let mut fixture = fixture();
                fixture.config = loaded.lock().unwrap().clone();
                Ok(fixture)
            },
        ));
        let mut first = SidebarIntent::of(&view.config);
        first.style.overrides.indent = Some(10.);
        view.commit_sidebar(first, cx);
        let mut second = SidebarIntent::of(&view.config);
        second.style.overrides.row_padding = Some(2.);
        view.commit_sidebar(second, cx);
        let mut preview = SidebarIntent::of(&view.config);
        preview.style.overrides.row_padding = Some(3.);
        view.preview_sidebar(preview, cx);
    });
    cx.run_until_parked();
    assert_eq!(
        *writes.lock().unwrap(),
        [
            Preference::SidebarIndent(Some(10.)),
            Preference::SidebarRowPadding(Some(2.))
        ]
    );
    view.read_with(cx, |view, _| {
        assert!(view.sidebar_dirty());
        assert_eq!(view.config.sidebar_style.overrides.row_padding, Some(3.));
    });
}

#[gpui::test]
fn failed_theme_resolution_does_not_drop_a_committed_sidebar_waiting_for_io(
    cx: &mut TestAppContext,
) {
    use crate::config::preferences::Preference;
    let (view, cx) = cx.add_window_view(skill_fixture);
    let writes = Arc::new(Mutex::new(Vec::new()));
    view.update(cx, |view, cx| {
        let captured = writes.clone();
        view.controls.preference_io = Some(controls::preferences::PreferenceIo::recording(
            move |edit| {
                captured.lock().unwrap().push(edit);
            },
        ));
        let mut theme_io = recording_themes(Arc::new(Mutex::new(Vec::new())), false);
        theme_io.resolve = Some(Arc::new(|_| Err(crate::Error::MissingHome)));
        view.theme_io = Some(theme_io);
        // Hold the persistence slot while the separate theme resolver completes.
        view.persistence.begin_save(Kind::Ordinary);
        let mut intent = SidebarIntent::of(&view.config);
        intent.style.overrides.indent = Some(10.);
        view.commit_sidebar(intent, cx);
        choose(view, "missing-definition", cx);
    });
    cx.run_until_parked();
    view.update(cx, |view, cx| {
        assert!(view.theme_load_failed);
        view.persistence.finish_save();
        view.finish_close(cx);
    });
    cx.run_until_parked();
    assert_eq!(
        *writes.lock().unwrap(),
        [Preference::SidebarIndent(Some(10.))]
    );
}
