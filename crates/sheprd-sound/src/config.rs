//! Herdr's `[ui.sound]` and toast delay, from the `config.toml` it writes.
//!
//! The file is read leniently ([`sheprd_lenient`]) and bounded: this crate
//! never owns it, and Herdr may write values this build cannot read.

use crate::{Error, Result};
use serde::Deserialize;
use sheprd_protocol::SemanticNotificationSound as Sound;
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

/// A settings file no larger than this is read; Herdr's is far smaller.
const MAX_FILE_BYTES: usize = 1_048_576;

#[derive(Default, Deserialize)]
#[serde(default)]
struct Document {
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    ui: Ui,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Ui {
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    sound: SoundConfig,
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    toast: Toast,
}

#[derive(Deserialize)]
#[serde(default)]
struct Toast {
    /// Herdr refuses a delay over an hour and keeps its default, so this does too.
    #[serde(deserialize_with = "delay")]
    delay_seconds: u64,
}

fn delay<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    Ok(sheprd_lenient::value(deserializer)?
        .filter(|delay: &u64| *delay <= 3600)
        .unwrap_or(1))
}
impl Default for Toast {
    fn default() -> Self {
        Self { delay_seconds: 1 }
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum AgentSetting {
    Default,
    On,
    Off,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    #[serde(deserialize_with = "sheprd_lenient::or_true")]
    enabled: bool,
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    path: Option<PathBuf>,
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    done_path: Option<PathBuf>,
    #[serde(deserialize_with = "sheprd_lenient::or_default")]
    request_path: Option<PathBuf>,
    #[serde(deserialize_with = "sheprd_lenient::entries")]
    agents: HashMap<String, AgentSetting>,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
            done_path: None,
            request_path: None,
            agents: HashMap::new(),
        }
    }
}

impl SoundConfig {
    /// Whether Herdr's settings let `agent`'s notifications play at all.
    pub fn allows(&self, agent: Option<&str>) -> bool {
        if !self.enabled {
            return false;
        }
        // Labels are a wire boundary. Unknown agents retain upstream's default.
        let mut label = agent.unwrap_or_default().trim().to_lowercase();
        for suffix in [".exe", ".cmd", ".bat", ".ps1", ".js"] {
            if label.ends_with(suffix) {
                label.truncate(label.len() - suffix.len());
                break;
            }
        }
        let label = label
            .rsplit(['/', '\\'])
            .find(|part| !part.is_empty())
            .unwrap_or_default();
        let key = match label {
            "claude" | "claude-code" => "claude",
            "cursor" | "cursor-agent" => "cursor",
            "devin" | "devin-cli" | "devin cli" => "devin", // i18n: data
            "agy" | "antigravity" | "antigravity-cli" => "agy",
            "cline" | ".cline" => "cline",
            "opencode" | "opencode2" | "open-code" => "open_code",
            "copilot" | "github-copilot" | "ghcs" => "github_copilot",
            "kimi" | "kimi-code" | "kimi code" => "kimi", // i18n: data
            "kiro" | "kiro-cli" => "kiro",
            "amp" | "amp-local" => "amp",
            "grok" | "grok-build" => "grok",
            "hermes" | "hermes-agent" => "hermes",
            "kilo" | "kilo-code" | "kilo code" => "kilo", // i18n: data
            "qodercli" | "qoderclicn" | "qoder" | "qodercn" => "qodercli",
            "qwen" | "qwen-code" | "qwen code" => "qwen", // i18n: data
            "letta" | "letta-code" | "letta code" => "letta", // i18n: data
            "muse" | "muse-code" | "muse-cli" => "muse",
            "pi" | "codex" | "gemini" | "droid" | "maki" => label,
            _ if label
                .strip_prefix("muse-bin-")
                .is_some_and(|s| s.starts_with(|c: char| c.is_ascii_digit())) =>
            {
                "muse"
            }
            _ => return true,
        };
        self.agents.get(key).copied().unwrap_or(if key == "droid" {
            AgentSetting::Off
        } else {
            AgentSetting::Default
        }) != AgentSetting::Off
    }
}

/// Herdr's sound settings, its toast delay, and the directory its sound paths
/// are relative to.
pub struct Settings {
    pub sound: SoundConfig,
    delay: u64,
    root: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: SoundConfig::default(),
            delay: 1,
            root: PathBuf::new(),
        }
    }
}

impl Settings {
    /// The settings in `path`, which is Herdr's `config.toml`; a missing file
    /// keeps every default.
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_path(path)
    }

    /// The settings in `path`, bounded and read leniently.
    fn load_path(path: &Path) -> Result<Self> {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(Error::from(error).at_path(path)),
        };
        let mut text = String::new();
        file.take(MAX_FILE_BYTES as u64 + 1)
            .read_to_string(&mut text)
            .map_err(|error| Error::from(error).at_path(path))?;
        if text.len() > MAX_FILE_BYTES {
            return Err(Error::ConfigSize.at_path(path));
        }
        Self::parse(&text, path).map_err(|error| error.at_path(path))
    }

    /// The settings `text` holds, with `path` as the file they came from.
    fn parse(text: &str, path: &Path) -> Result<Self> {
        let document: Document = toml::from_str(text)?;
        Ok(Self {
            sound: document.ui.sound,
            delay: document.ui.toast.delay_seconds,
            root: path.parent().unwrap_or_else(|| Path::new(".")).to_owned(),
        })
    }

    /// How long Herdr waits before it sounds a notification, in seconds.
    pub fn ui_delay(&self) -> u64 {
        self.delay
    }

    /// The file Herdr's settings name for `sound`, relative to the settings
    /// file, and `None` when they name none.
    pub fn path_for(&self, sound: Sound) -> Option<PathBuf> {
        let path = match sound {
            Sound::Done => self.sound.done_path.as_ref(),
            Sound::Request => self.sound.request_path.as_ref(),
        }
        .or(self.sound.path.as_ref())?;
        Some(self.root.join(path))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
