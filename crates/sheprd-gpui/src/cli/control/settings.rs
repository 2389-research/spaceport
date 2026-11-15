//! `config`, `theme`, `font`, `keybinds`, `caffeine`, `sound`,
//! `integrations`, and `language`.
use super::{Args, CliError, Sound, Switch, duration, switch, unknown};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigCommand {
    /// Where the settings files are.
    Path,
    /// A setting's value, by its dotted key.
    Get(String),
    /// Change a setting in the local overrides file; the app picks it up.
    Set {
        key: String,
        value: String,
    },
    /// Remove a setting from the local overrides file.
    Unset(String),
    Reload,
    Check,
}

/// `theme` and `font`: list what there is, or choose one. A font may be
/// unset, going back to the default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeCommand {
    List,
    Set(Option<String>),
}

/// `language`: what the windows show, the languages there are, or the
/// `language` setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageCommand {
    Status,
    List,
    Set(sheprd_i18n::Language),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeybindsCommand {
    List,
    Check,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaffeineCommand {
    Status,
    Set {
        switch: Switch,
        seconds: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCommand {
    List,
    Play(Sound),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrationsCommand {
    List,
    InstallSkill,
}

/// A dotted key of plain TOML words, such as `terminal.size`.
fn key(text: String) -> Result<String, CliError> {
    let valid = !text.is_empty()
        && text.len() <= 256
        && text.split('.').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        });
    valid.then_some(text).ok_or(CliError::InvalidValue("KEY"))
}

pub(super) const CONFIG_USAGE: &str = "spaceport config [path]\n    The settings files.\nspaceport config get KEY\n    The setting's value, as TOML: your override, else the default.\nspaceport config set KEY VALUE\nspaceport config unset KEY\n    Change a setting in spaceport/config.local.toml. VALUE is read as TOML\n    (14, true, [\"a\"], { show = true }), else as text. A value or key\n    Spaceport would not load is refused and the file left as it was.\n    A running Spaceport applies the change at once. KEY is dotted, such\n    as terminal.size. Unsetting a key that is not set exits 4.\nspaceport config check\n    Report settings that do not load and unknown keys, and, when Spaceport\n    runs, fonts that are not installed; exits 1 when there are any.\nspaceport config reload\n    Reread Spaceport's settings files.\nOnly reload needs Spaceport running.";

pub(super) fn config(args: &mut Args) -> Result<ConfigCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(ConfigCommand::Path);
    };
    Ok(match verb.to_str() {
        Some("path") => ConfigCommand::Path,
        Some("get") => ConfigCommand::Get(key(args.required_text("KEY")?)?),
        Some("set") => ConfigCommand::Set {
            key: key(args.required_text("KEY")?)?,
            value: args.words()?.ok_or(CliError::MissingArgument {
                command: "config set",
                what: "VALUE",
            })?,
        },
        Some("unset") => ConfigCommand::Unset(key(args.required_text("KEY")?)?),
        Some("reload") => ConfigCommand::Reload,
        Some("check") => ConfigCommand::Check,
        _ => return Err(unknown("config", verb)),
    })
}

pub(super) const THEME_USAGE: &str = "spaceport theme [list]\n    The themes there are; * marks the selected one.\nspaceport theme set NAME...\n    Choose and save a theme, or light:NAME,dark:NAME to follow the\n    system appearance. Saved even when Spaceport is not running.";
pub(super) const FONT_USAGE: &str = "spaceport font [list]\n    The installed font families.\nspaceport font set NAME...\nspaceport font reset\n    The font of every face; reset goes back to the default.\nThese need Spaceport running, which knows the installed fonts.";

pub(super) fn theme(args: &mut Args) -> Result<ThemeCommand, CliError> {
    let family = args.name();
    let Some(verb) = args.verb() else {
        return Ok(ThemeCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => ThemeCommand::List,
        Some("set") => ThemeCommand::Set(Some(args.words()?.ok_or(CliError::MissingArgument {
            command: family,
            what: "NAME",
        })?)),
        Some("reset") if family == "font" => ThemeCommand::Set(None),
        _ => return Err(unknown(family, verb)),
    })
}

pub(super) const KEYBINDS_USAGE: &str = "spaceport keybinds [list]\n    The window's shortcuts.\nspaceport keybinds check\n    Herdr's custom commands this client cannot reach; exits 1 if any.";

pub(super) fn keybinds(args: &mut Args) -> Result<KeybindsCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(KeybindsCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => KeybindsCommand::List,
        Some("check") => KeybindsCommand::Check,
        _ => return Err(unknown("keybinds", verb)),
    })
}

pub(super) const CAFFEINE_USAGE: &str = "spaceport caffeine [status]\nspaceport caffeine on|off|toggle [--for 90s|30m|2h]\n    Keep the display and the machine awake. With --for (at most 24h),\n    caffeine turns off then, unless it was changed in the meantime.";

pub(super) fn caffeine(args: &mut Args) -> Result<CaffeineCommand, CliError> {
    let seconds = args
        .text_option("--for")?
        .map(|text| duration(&text).ok_or(CliError::InvalidValue("--for (90s, 30m, 2h)")))
        .transpose()?;
    let verb = args.verb();
    if verb.as_ref().is_some_and(|verb| verb == "status") {
        return Ok(CaffeineCommand::Status);
    }
    if verb.is_none() && seconds.is_none() {
        return Ok(CaffeineCommand::Status);
    }
    let switch = switch(verb)?;
    if seconds.is_some() && switch == Switch::Off {
        return Err(CliError::InvalidValue("--for with on or toggle"));
    }
    Ok(CaffeineCommand::Set { switch, seconds })
}

pub(super) const SOUND_USAGE: &str = "spaceport sound [list]\n    The notification sounds.\nspaceport sound play [done|request]\n    Play one now, from the file Herdr's sound settings name or the\n    built-in one.";

pub(super) fn sound(args: &mut Args) -> Result<SoundCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(SoundCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => SoundCommand::List,
        Some("play") => {
            SoundCommand::Play(match args.optional_text()?.as_deref().unwrap_or("done") {
                "done" => Sound::Done,
                "request" => Sound::Request,
                _ => return Err(CliError::InvalidValue("done|request")),
            })
        }
        _ => return Err(unknown("sound", verb)),
    })
}

pub(super) const INTEGRATIONS_USAGE: &str = "spaceport integrations [list]\n    The agent integrations Herdr offers the window's host. Install them\n    from Spaceport's integrations settings.\nspaceport integrations install-skill\n    Install or update the agent skill for Claude Code and other agents.\n    Runs here; Spaceport need not run.";

pub(super) fn integrations(args: &mut Args) -> Result<IntegrationsCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(IntegrationsCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => IntegrationsCommand::List,
        Some("install-skill") => IntegrationsCommand::InstallSkill,
        _ => return Err(unknown("integrations", verb)),
    })
}

pub(super) const LANGUAGE_USAGE: &str = "spaceport language [status]\n    The language set, and the locale Spaceport shows: the best match for\n    the setting among the languages there are, else en-US.\nspaceport language list\n    The languages there are, each named in itself, and the en-XA and en-XB\n    pseudo-locales.\nspaceport language set system|TAG\n    Save the language setting: system follows the OS's languages; TAG is\n    a BCP 47 tag such as fr-CA. A running Spaceport switches at once.\nOnly status asks Spaceport; without it, status reports what it would show.";

pub(super) fn language(args: &mut Args) -> Result<LanguageCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(LanguageCommand::Status);
    };
    Ok(match verb.to_str() {
        Some("status") => LanguageCommand::Status,
        Some("list") => LanguageCommand::List,
        Some("set") => {
            LanguageCommand::Set(args.required_text("TAG")?.parse().map_err(|_| {
                CliError::InvalidValue("TAG (system or a BCP 47 tag such as fr-CA)")
            })?)
        }
        _ => return Err(unknown("language", verb)),
    })
}
