// ABOUTME: Exercises final preference snapshots through owner drop and app shutdown.
// ABOUTME: Uses real files and the app's quit boundary instead of polling for persistence.
use super::{Chrome, Preferences, TestDirectory, await_loaded, fs, read_chrome};
use std::sync::Arc;

#[gpui::test]
fn quitting_after_the_owner_drops_keeps_the_latest_preferences(cx: &mut gpui::TestAppContext) {
    let directory = TestDirectory::new();
    let path = directory.0.join("preferences.json");
    let mut preferences = cx.update(|cx| Preferences::start_for_app(Ok(path.clone()), cx));
    await_loaded(&mut preferences);
    for width in 1..=256 {
        preferences.save(Chrome {
            sidebar_width: Some(width as f32),
            ..Chrome::default()
        });
    }
    let worker = Arc::downgrade(preferences.worker.as_ref().unwrap());
    drop(preferences);
    cx.update(|cx| cx.shutdown());
    let saved = read_chrome(&path).unwrap_or_default();
    // Drain even the failing baseline before removing its private directory.
    if let Some(worker) = worker.upgrade() {
        worker.finish();
    }
    assert_eq!(saved.sidebar_width, Some(256.0));
    assert!(
        fs::read_dir(&directory.0)
            .unwrap()
            .all(|entry| entry.unwrap().path() == path)
    );
}

#[gpui::test]
fn quitting_with_a_live_owner_finishes_before_further_saves(cx: &mut gpui::TestAppContext) {
    let directory = TestDirectory::new();
    let path = directory.0.join("preferences.json");
    let mut preferences = cx.update(|cx| Preferences::start_for_app(Ok(path.clone()), cx));
    await_loaded(&mut preferences);
    preferences.save(Chrome {
        sidebar_width: Some(360.0),
        ..Chrome::default()
    });
    cx.update(|cx| cx.shutdown());
    assert_eq!(read_chrome(&path).unwrap().sidebar_width, Some(360.0));
    preferences.save(Chrome::default());
    drop(preferences);
    assert_eq!(read_chrome(&path).unwrap().sidebar_width, Some(360.0));
}

#[gpui::test]
fn completed_preference_workers_remove_their_quit_observers(cx: &mut gpui::TestAppContext) {
    let directory = TestDirectory::new();
    for index in 0..8 {
        let path = directory.0.join(format!("preferences-{index}.json"));
        let mut preferences = cx.update(|cx| Preferences::start_for_app(Ok(path), cx));
        await_loaded(&mut preferences);
        let worker = preferences.worker.as_ref().unwrap().clone();
        drop(preferences);
        worker.finish();
        cx.run_until_parked();
        assert_eq!(
            Arc::weak_count(&worker),
            0,
            "completed worker retained its app quit observer"
        );
    }
}
