use crate::{
    SpaceportWindow,
    config::{Config, ThemeName},
};
use chrono::{DateTime, Utc};
use gpui::*;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    mpsc::{self, Receiver, TryRecvError},
};

mod worker;
use worker::Worker;

impl SpaceportWindow {
    /// Write one setting off the UI thread, then reload so the window shows
    /// what the file now says rather than what was clicked.
    pub(crate) fn save_preference(
        &mut self,
        save: impl FnOnce() -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.theme_save_in_flight() {
            return;
        }
        if self.config_load.is_some() {
            // A combined write and reload would be refused here, discarding
            // the click. The config lock serializes the write with other
            // saves, and the config watcher reloads once the busy load ends.
            self.write_preference(save, cx);
            return;
        }
        let light = crate::app::light_appearance(cx);
        let text_system = cx.text_system().clone();
        self.load_gui_config_with(
            move || {
                save()?;
                let mut config = Config::load()?;
                config.resolve_fonts(|| text_system.all_font_names());
                let keymap = crate::config::keymap(&config)?;
                let theme = if ThemeName::follows_herdr(&config.theme, light) {
                    Default::default()
                } else {
                    config.theme(light)?
                };
                Ok((config, keymap, theme))
            },
            cx,
        );
    }

    pub(crate) fn write_preference(
        &mut self,
        save: impl FnOnce() -> crate::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let saved = cx.background_executor().spawn(async move { save() });
        cx.spawn(async move |this, cx| {
            let result = saved.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(()) => this.load_gui_config(cx),
                Err(error) => {
                    tracing::warn!(%error, "Could not save GUI preference");
                    this.notices.show(
                        crate::notice::Cause::Configuration,
                        format!("Save GUI config: {error}"),
                    );
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

pub use spaceport_settings::AgentSort;

/// The window chrome this endpoint remembers between runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chrome {
    /// The activity bar's panel; `None` before one was picked.
    pub activity_panel: Option<crate::sidebar::ActivityPanel>,
    /// When each activity bar section was last open, so its count shows in
    /// the accent only for what changed since.
    pub activity_last_opened: HashMap<crate::sidebar::ActivityPanel, DateTime<Utc>>,
    pub sidebar_width: Option<f32>,
    pub sidebar_split: Option<f32>,
    /// The sort the user picked with the panel toggle, like upstream's
    /// `agent_panel_sort` preference. `None` follows the daemon's config.
    pub agent_sort: Option<AgentSort>,
    /// The device IDs in the order the user dragged them into.
    pub device_order: Vec<String>,
}

/// Endpoint-local preferences, saved on one coalescing worker. Dropping
/// never waits for the disk, and app quit writes the latest pending value.
pub struct Preferences {
    loaded: Option<Receiver<Chrome>>,
    worker: Option<Arc<Worker>>,
}

impl Preferences {
    pub fn new(socket: &Path, cx: &App) -> Self {
        Self::start_for_app(
            state_dir()
                .map(|dir| endpoint_path(&dir, socket))
                .ok_or(crate::Error::MissingStateRoot),
            cx,
        )
    }

    pub(crate) fn for_core(core_id: &str, cx: &App) -> Self {
        Self::start_for_app(
            state_dir()
                .map(|dir| core_path(&dir, core_id))
                .ok_or(crate::Error::MissingStateRoot),
            cx,
        )
    }

    fn start_for_app(path: crate::Result<PathBuf>, cx: &App) -> Self {
        let preferences = Self::start(path);
        if let Some(worker) = &preferences.worker {
            let completed = worker.take_completion();
            let worker = Arc::downgrade(worker);
            // App-owned: the final window can close before quit observers run.
            let subscription = cx.on_app_quit(move |cx| {
                let worker = worker.upgrade();
                cx.background_executor().spawn(async move {
                    if let Some(worker) = worker {
                        worker.finish();
                    }
                })
            });
            // Subscriptions are UI-owned. Remove this one as soon as its worker drains.
            cx.spawn(async move |_| {
                if let Some(completed) = completed {
                    let _ = completed.await;
                }
                drop(subscription);
            })
            .detach();
        }
        preferences
    }

    fn start(path: crate::Result<PathBuf>) -> Self {
        let (loaded_tx, loaded) = mpsc::sync_channel(1);
        let worker = match path {
            Ok(path) => match Worker::start(path, loaded_tx) {
                Ok(worker) => Some(worker),
                Err(error) => {
                    tracing::warn!(category = "preferences_worker_start", error_kind = ?error.kind(), "Cannot start GPUI preferences worker");
                    None
                }
            },
            Err(_) => {
                tracing::warn!(
                    category = "preferences_location",
                    "Cannot locate GPUI preferences"
                );
                let _ = loaded_tx.send(Chrome::default());
                None
            }
        };
        Self {
            loaded: Some(loaded),
            worker,
        }
    }

    /// Takes the initial result once; `None` means pending or already taken.
    /// Defaults stand in for a failed initial load.
    pub fn loaded(&mut self) -> Option<Chrome> {
        let result = match self.loaded.as_ref()?.try_recv() {
            Ok(chrome) => Some(chrome),
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Some(Chrome::default()),
        };
        self.loaded = None;
        result
    }

    /// Queues the whole chrome, so saving one field never drops the others.
    /// A later save replaces one the worker has not written yet.
    pub fn save(&self, chrome: Chrome) {
        if let Some(worker) = &self.worker {
            worker.save(chrome);
        }
    }
}

impl Drop for Preferences {
    fn drop(&mut self) {
        // Close without joining: the worker writes what is pending while the
        // app runs, and the quit observer finishes it if the app quits first.
        if let Some(worker) = &self.worker {
            worker.stop();
        }
    }
}

pub(crate) use spaceport_client::local_path::state_dir;

fn endpoint_path(dir: &Path, socket: &Path) -> PathBuf {
    let hash = hash_bytes(socket.as_os_str().as_encoded_bytes());
    dir.join(format!("local-{hash:016x}.json"))
}

fn core_path(dir: &Path, core_id: &str) -> PathBuf {
    let hash = hash_bytes(core_id.as_bytes());
    dir.join(format!("core-{hash:016x}.json"))
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn read_chrome(path: &Path) -> crate::Result<Chrome> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Chrome::default()),
        Err(error) => return Err(error.into()),
    };
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    let object = value
        .as_object()
        .ok_or(crate::config::Error::PreferencesNotObject)?;
    // Earlier builds always wrote `agent_sort`, so its default value cannot be
    // told from a choice; only "priority" proves the user toggled it.
    let agent_sort = AgentSort::parse(object.get("agent_sort_manual")).or_else(|| {
        AgentSort::parse(object.get("agent_sort")).filter(|sort| *sort == AgentSort::Priority)
    });
    let sidebar_width = match object.get("sidebar_width_px") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            let width = value.as_f64().map(|width| width as f32);
            match width {
                Some(width) if width.is_finite() && width > 0.0 => Some(width),
                _ => return Err(crate::Error::InvalidStoredWidth),
            }
        }
    };
    let sidebar_split = object
        .get("sidebar_split")
        .and_then(serde_json::Value::as_f64)
        .map(|split| split as f32)
        .filter(|split| split.is_finite() && (0.1..=0.9).contains(split));
    Ok(Chrome {
        activity_panel: object
            .get("activity_panel")
            .and_then(serde_json::Value::as_str)
            .and_then(retained_activity_panel),
        activity_last_opened: last_opened(object.get("activity_last_opened_at")),
        sidebar_width,
        sidebar_split,
        agent_sort,
        device_order: device_order(object.get("device_order")),
    })
}

/// Old Activity choices have no section after Orchestrator is retired.
/// Reading them changes only the in-memory choice, not the saved file.
fn retained_activity_panel(name: &str) -> Option<crate::sidebar::ActivityPanel> {
    use crate::sidebar::ActivityPanel;
    if matches!(name, "pull-requests" | "issues" | "tasks") {
        return Some(ActivityPanel::Workspaces);
    }
    ActivityPanel::parse(name).map(|panel| match panel {
        ActivityPanel::Workspaces | ActivityPanel::Usage => panel,
        ActivityPanel::Overview | ActivityPanel::Automations => ActivityPanel::Workspaces,
    })
}

/// When each section was last open. An unknown section or unreadable time is
/// forgotten rather than failing the whole file; there are few sections.
fn last_opened(
    value: Option<&serde_json::Value>,
) -> HashMap<crate::sidebar::ActivityPanel, DateTime<Utc>> {
    let mut visits = HashMap::new();
    for (name, at) in value
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
    {
        let Some((panel, at)) = retained_activity_panel(name).zip(
            at.as_str()
                .and_then(|at| DateTime::parse_from_rfc3339(at).ok()),
        ) else {
            continue;
        };
        let at = at.with_timezone(&Utc);
        visits
            .entry(panel)
            .and_modify(|latest: &mut DateTime<Utc>| *latest = (*latest).max(at))
            .or_insert(at);
    }
    visits
}

/// The stored device order. Anything but a list of IDs is forgotten rather
/// than failing the whole file, and the list is bounded like the catalogs.
fn device_order(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .filter(|id| !id.is_empty() && id.len() <= MAX_DEVICE_ID)
        .take(MAX_DEVICES)
        .map(str::to_owned)
        .collect()
}

/// Bounds on the stored device order: far more devices than anyone saves,
/// each ID far longer than a catalog key.
const MAX_DEVICES: usize = 1024;
const MAX_DEVICE_ID: usize = 1024;

impl Serialize for Chrome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error as _;
        let width = self.sidebar_width;
        if width.is_some_and(|width| !width.is_finite() || width <= 0.0) {
            return Err(S::Error::custom(crate::Error::InvalidSidebarWidth));
        }
        let last_opened: serde_json::Map<String, serde_json::Value> = self
            .activity_last_opened
            .iter()
            .map(|(panel, at)| (panel.name().to_owned(), at.to_rfc3339().into()))
            .collect();
        serde_json::json!({
            "activity_panel": self.activity_panel.map(crate::sidebar::ActivityPanel::name),
            "activity_last_opened_at": last_opened,
            "sidebar_width_px": width,
            "sidebar_split": self.sidebar_split.filter(|split| {
                split.is_finite() && (0.1..=0.9).contains(split)
            }),
            "agent_sort_manual": self.agent_sort.map(|sort| sort.to_string()),
            "device_order": self.device_order,
        })
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests;

// A sibling of `tests`: its glob import shadows `#[test]` with GPUI's macro.
#[cfg(test)]
mod busy_load_tests {
    #[gpui::test]
    #[allow(clippy::unwrap_used)]
    fn preference_clicked_during_a_config_load_is_still_written(cx: &mut gpui::TestAppContext) {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        let written = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        view.update(cx, |view, cx| {
            // A load that never finishes, such as one the watcher started.
            view.config_load = Some(cx.spawn(async |_, _| std::future::pending::<()>().await));
            let written = written.clone();
            view.save_preference(
                move || {
                    written.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                },
                cx,
            );
        });
        cx.run_until_parked();
        assert_eq!(written.load(std::sync::atomic::Ordering::SeqCst), 1);

        view.update(cx, |view, cx| {
            view.save_preference(
                || Err(crate::config::Error::InvalidTerminalPadding("padding_x").into()),
                cx,
            );
        });
        cx.run_until_parked();
        view.read_with(cx, |view, _| {
            let error = view.notices.text().unwrap();
            assert!(error.starts_with("Save GUI config:"), "{error}");
            // The busy load is left alone; the watcher applies the write.
            assert!(view.config_load.is_some());
        });
    }
}
