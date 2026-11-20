// ABOUTME: Shares isolated Settings fixtures and persistence writers.
// ABOUTME: Registers regression topics for the native preferences window.
use super::*;
use crate::config::FontFace;
use core::prelude::v1::test;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

mod activity_bar_windows;
mod appearance_messages;
mod catalog_rows;
mod config_watch;
mod custom_keyboard;
mod en_xa;
mod font_scale;
mod host_history_reload;
mod keyboard;
mod language_picker;
mod language_reload;
#[cfg(windows)]
mod launch_at_login;
mod layout_drafts;
mod load_save;
mod mixed_follow;
mod navigation_groups;
mod navigation_resize;
mod navigation_scale;
mod navigation_scroll;
mod page_messages;
mod pages_behaviour;
mod quit_saves;
mod removed_plugins;
mod removed_worktree_cleanup;
mod rtl;
mod save_intents;
mod save_now;
mod scrollbars;
mod search;
mod section_order;
#[cfg(unix)]
mod shared_save_queue;
mod sidebar_drafts;
mod system_contrast;
mod tab_layout;
mod theme_drafts;
// Saving a shared Herdr theme is Unix-only.
mod theme_hover;
mod theme_mode;
#[cfg(unix)]
mod theme_recovery;
mod theme_sources;
mod window_commands;
mod window_lifecycle;

fn fixture_load() -> crate::Result<Loaded> {
    Ok(fixture())
}

type SizeWrites = Arc<Mutex<Vec<Vec<(FontFace, f32)>>>>;

fn recording_sizes(writes: SizeWrites) -> SizeIo {
    SizeIo {
        write: Arc::new(move |sizes| {
            writes.lock().unwrap().push(sizes);
            Ok(())
        }),
        load: fixture_load,
    }
}

fn recording_themes(writes: Arc<Mutex<Vec<String>>>, fail: bool) -> themes::ThemeIo {
    let disk = Arc::new(Mutex::new("Default".to_owned()));
    let saved = disk.clone();
    themes::ThemeIo {
        shared: None,
        resolve: None,
        write: Arc::new(move |name, shared| {
            assert!(shared.is_none());
            writes.lock().unwrap().push(name.clone());
            if fail {
                return Err(crate::Error::MissingHome);
            }
            *saved.lock().unwrap() = name;
            Ok(())
        }),
        load: Arc::new(move || {
            let mut loaded = fixture();
            loaded.config.theme = disk.lock().unwrap().clone();
            loaded.theme = loaded.config.theme(false)?;
            Ok(loaded)
        }),
    }
}

fn recording_layouts(
    writes: Arc<Mutex<Vec<crate::config::LayoutMode>>>,
    fail: bool,
) -> layouts::LayoutIo {
    let disk = Arc::new(Mutex::new(Config::default().layout.mode));
    let saved = disk.clone();
    layouts::LayoutIo {
        write: Arc::new(move |mode| {
            writes.lock().unwrap().push(mode);
            if fail {
                return Err(crate::Error::MissingHome);
            }
            *saved.lock().unwrap() = mode;
            Ok(())
        }),
        load: Arc::new(move || {
            let mut loaded = fixture();
            loaded.config.layout.mode = *disk.lock().unwrap();
            Ok(loaded)
        }),
    }
}

fn choose(view: &mut SettingsWindow, name: &str, cx: &mut Context<SettingsWindow>) {
    view.accept_theme_choice(
        themes::Choice {
            scope: themes::Scope::App,
            name: name.into(),
        },
        cx,
    );
}

pub(super) fn fixture() -> Loaded {
    Loaded {
        config: Config::default(),
        keymap: Default::default(),
        theme: Theme::default(),
        shared: None,
        error: None,
    }
}

fn open_fixture(source: WeakEntity<SheprdWindow>, cx: &mut App) {
    open_with(source, cx, |_, _, _| {});
}

pub(super) fn skill_fixture(
    window: &mut Window,
    cx: &mut Context<SettingsWindow>,
) -> SettingsWindow {
    let source = cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx));
    let mut view = fixture_new(source.downgrade(), cx);
    view.section = Section::General;
    view
}

pub(super) fn fixture_new(
    source: WeakEntity<SheprdWindow>,
    cx: &mut Context<SettingsWindow>,
) -> SettingsWindow {
    let mut view = SettingsWindow::new(source, cx);
    view.host_history.read = Arc::new(|_, _| Err(crate::Error::MissingHome));
    view
}

pub(super) fn skill_load() -> crate::Result<Loaded> {
    Ok(Loaded {
        config: Config::default(),
        keymap: Default::default(),
        theme: Default::default(),
        shared: None,
        error: None,
    })
}

/// GPUI's keystroke convenience helper omits key releases.
pub(in crate::settings_window) fn press_key(cx: &mut VisualTestContext, key: &str) {
    let keystroke = crate::keymap::parse_keystroke(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}
