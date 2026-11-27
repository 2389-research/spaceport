//! Why the user's settings or a theme could not be loaded, validated, or edited.

use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    /// A configuration file that could not be read at the URI given.
    #[error("{source}")]
    ConfigFile {
        uri: Option<String>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("{0}")]
    ConfigLoad(#[source] config_loader::ConfigError),
    /// A `[devices.<id>]` theme that does not parse, named by its device.
    #[error("devices.{device}.theme: {source}")]
    DeviceTheme {
        device: String,
        #[source]
        source: Box<Error>,
    },
    #[error("{0}.fallback families must not be empty")]
    EmptyFontFallback(&'static str),
    #[error("{0}.family must not be empty")]
    EmptyFontFamily(&'static str),
    #[error("theme must not be empty")]
    EmptyTheme,
    /// Herdr's shared settings, which the app reads and edits beside its own.
    /// Not `transparent`: this error must stay in the source chain, where the
    /// caller reads which shared-config failure it was.
    #[error("{0}")]
    HerdrSettings(#[from] crate::herdr_settings::Error),
    /// A `[backdrop]` value the window cannot paint with.
    #[error("{0}")]
    InvalidBackdrop(#[from] BackdropProblem),
    #[error("[chat.mirrors] needs at most 16 valid pane and endpoint identifiers")]
    InvalidChatMirror,
    #[error("chat.max_width must be 0 (full width) or 320–4000 logical pixels")]
    InvalidChatWidth,
    #[error("devices.{0:?} is not a saved device ID")]
    InvalidDeviceId(String),
    #[error("devices and each devices.<id> must be tables")]
    InvalidDevicesTable,
    #[error("favorite_themes must be a list of theme names")]
    InvalidFavoriteThemes,
    #[error("{0}.size must be finite and between 8 and 48 logical pixels")]
    InvalidFontSize(&'static str),
    #[error("{0}.weight must be a weight name such as \"medium\" or a number from 100 to 900")]
    InvalidFontWeight(&'static str),
    #[error("sidebar.hosts.{host:?} must be a #rgb or #rrggbb colour, not {value:?}")]
    InvalidHostColor { host: String, value: String },
    #[error("{0}.line_height must be a finite number between 1 and 2 times the font size")]
    InvalidLineHeight(&'static str),
    #[error("Project paths must be absolute UTF-8 paths without control characters")]
    InvalidProjectRoots,
    #[error("layout.sidebar_gap must be finite and between 0 and 64 logical pixels")]
    InvalidSidebarGap,
    #[error("sidebar.{key} must be finite and between 0 and {max} logical pixels")]
    InvalidSidebarMetric { key: &'static str, max: f32 },
    #[error("sound.volume must be between 0 and 100")]
    InvalidSoundVolume,
    #[error(
        "terminal.{0} must be one number or a pair of numbers, each finite and between 0 and 64 logical pixels"
    )]
    InvalidTerminalPadding(&'static str),
    #[error("a theme that follows the system must name both sides: light:NAME,dark:NAME")]
    InvalidThemePair,
    #[error("theme must be a name, absolute path, or ~/ path")]
    InvalidThemePath,
    #[error("worktrees.tint must be a #rgb or #rrggbb colour, not {0:?}")]
    InvalidWorktreeTint(String),
    #[error("zen.font_size must be finite and between 8 and 48 logical pixels")]
    InvalidZenFontSize,
    #[error("zen.width must be 20 to 1000 columns, or a percentage from \"10%\" to \"100%\"")]
    InvalidZenWidth,
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{key}: a part of this key holds a value, not a table; unset that part first")]
    KeyNotTable { key: String },
    #[error("Cannot migrate {}: {} already contains different settings. Merge your old settings into the local file, then remove the old config; it will be regenerated.", original.display(), local.display())]
    MigrationConflict { original: PathBuf, local: PathBuf },
    #[error("HOME is not set")]
    MissingHome,
    #[error("{}: {source}", path.display())]
    Path {
        path: PathBuf,
        #[source]
        source: Box<Error>,
    },
    #[error("preferences must be an object")]
    PreferencesNotObject,
    #[error("XDG_CONFIG_HOME must be an absolute path")]
    RelativeConfigRoot,
    /// A `[ssh]` table this build cannot use.
    #[error("{0}")]
    SshHosts(#[from] crate::ssh_hosts::Error),
    #[error("line {line}: {key}: {source}")]
    ThemeLine {
        line: usize,
        key: String,
        #[source]
        source: ThemeParseError,
    },
    #[error("theme {name:?} not found in {directories:?}")]
    ThemeNotFound {
        name: String,
        directories: Vec<PathBuf>,
    },
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    #[error("{0}")]
    TomlEdit(#[from] toml_edit::TomlError),
    #[error("[devices] must list at most {0} devices")]
    TooManyDevices(usize),
    #[error("{0}.fallback must list at most 8 families")]
    TooManyFontFallbacks(&'static str),
    #[error("keybindings.{0} is not a command; see the keybindings list in spaceport/config.toml")]
    UnknownKeybinding(String),
    #[error(
        "Spaceport has no setting {}; spaceport/config.toml lists every setting",
        .0.join(", ")
    )]
    UnknownKeys(Vec<String>),
    #[error("Unknown sidebar layout {0:?}")]
    UnknownLayout(String),
    // keep-sorted end
}

/// Why a `[backdrop]` value cannot be used.
#[derive(Debug, thiserror::Error)]
pub enum BackdropProblem {
    #[error("backdrop.image must not be empty")]
    EmptyImage,
    #[error("backdrop.image must be a UTF-8 path")]
    ImagePathEncoding,
    #[error("backdrop.opacity must be finite and between 0 and 1")]
    Opacity,
    #[error("backdrop.image must be an absolute path or start with ~/")]
    RelativeImage,
}

/// Why one line of a terminal theme's colors did not parse.
#[derive(Debug, thiserror::Error)]
pub enum ThemeParseError {
    #[error("expected a six-digit RGB hex color (optionally prefixed by #)")]
    InvalidColor,
    #[error("invalid hex color")]
    InvalidHex(#[source] std::num::ParseIntError),
    #[error("expected index=color")]
    MissingPaletteColor,
    #[error("palette index must be between 0 and 255")]
    InvalidPaletteIndex(#[source] std::num::ParseIntError),
    #[error("palette index must be between 0 and 255")]
    PaletteIndexOutOfRange,
}

impl From<config_loader::ConfigError> for Error {
    fn from(error: config_loader::ConfigError) -> Self {
        // config preserves this cause but does not expose it through Error::source.
        match error {
            config_loader::ConfigError::FileParse { uri, cause } => {
                Self::ConfigFile { uri, source: cause }
            }
            error => Self::ConfigLoad(error),
        }
    }
}

impl Error {
    /// This failure, as one concerning `path`.
    pub fn at_path(self, path: &Path) -> Self {
        Self::Path {
            path: path.to_owned(),
            source: Box::new(self),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
