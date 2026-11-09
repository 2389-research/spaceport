mod args;
pub mod control;

use sheprd_client::ConnectTarget;
use std::ffi::OsString;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LaunchMode {
    #[default]
    Normal,
    Help,
    BuildInfo,
    /// Talk to the running app, or read its files, then exit without
    /// starting GPUI.
    Control(control::Invocation),
    /// Hand a `spaceport://` link to the running app, or start the app
    /// and follow it there.
    Link(crate::link::Link),
    #[cfg(feature = "integration-test")]
    Integration,
    #[cfg(feature = "integration-test")]
    Sidebar,
    #[cfg(feature = "integration-test")]
    Performance,
}

#[derive(Debug)]
pub struct LaunchOptions {
    pub target: ConnectTarget,
    pub mode: LaunchMode,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CliError {
    #[error("--socket requires a path")]
    MissingSocketPath,
    #[error("--socket may only be specified once")]
    DuplicateSocket,
    #[error("Unknown option: {}", .0.to_string_lossy())]
    UnknownOption(OsString),
    #[error("{0} requires a value")]
    MissingValue(&'static str),
    #[error("{0} may only be specified once")]
    DuplicateOption(&'static str),
    #[error("Invalid {0}")]
    InvalidValue(&'static str),
    #[error("{command} requires {what}")]
    MissingArgument {
        command: &'static str,
        what: &'static str,
    },
    #[error("{command}: unexpected {}", argument.to_string_lossy())]
    Unexpected {
        command: &'static str,
        argument: OsString,
    },
    #[error("Unknown {family} command: {}", command.to_string_lossy())]
    UnknownCommand {
        family: &'static str,
        command: OsString,
    },
    #[error("arguments must be UTF-8")]
    InvalidEncoding(OsString),
    #[error("invalid link: {0}")]
    Link(#[from] crate::link::Error),
    #[error("a spaceport:// link must be the only argument")]
    LinkArguments,
    #[cfg(feature = "integration-test")]
    #[error("native test modes are mutually exclusive and may only be specified once")]
    ConflictingTestModes,
    #[cfg(feature = "integration-test")]
    #[error("fixture tests cannot be combined with connection options or --integration-test")]
    ConflictingFixtureOptions,
    #[cfg(feature = "integration-test")]
    #[error("--integration-test requires an explicit --socket")]
    MissingIntegrationSocket,
    #[cfg(all(feature = "integration-test", not(any(target_os = "macos", windows))))]
    #[error("--performance-test currently requires macOS or Windows native event delivery")]
    UnsupportedPerformancePlatform,
}

// The framed record is also read without execution by cross-platform packaging.
pub fn build_info() -> &'static str {
    const PREFIX_LEN: usize = "\0HERDR_BUILD_IDENTITY_V1\n".len();
    const RECORD: &str = concat!(
        "\0HERDR_BUILD_IDENTITY_V1\n",
        "worktree=",
        env!("HERDR_BUILD_WORKTREE"),
        "\n",
        "branch=",
        env!("HERDR_BUILD_BRANCH"),
        "\n",
        "pr=",
        env!("HERDR_BUILD_PR"),
        "\n\0"
    );
    let record = std::hint::black_box(RECORD);
    &record[PREFIX_LEN..record.len() - 1]
}

impl LaunchOptions {
    pub fn parse(args: impl IntoIterator<Item = impl Into<OsString>>) -> Result<Self, CliError> {
        let mut args = args.into_iter().map(Into::into).peekable();
        if let Some(family) = args.next_if(control::is_family) {
            return Ok(Self {
                target: ConnectTarget::Local,
                mode: LaunchMode::Control(control::parse(family, args)?),
            });
        }
        if let Some(link) = args.next_if(|arg| crate::link::is_link(arg)) {
            // The shell substitutes the whole link into one argument; a
            // second one means it was split, perhaps to smuggle an option.
            if args.next().is_some() {
                return Err(CliError::LinkArguments);
            }
            let link = link.into_string().map_err(CliError::InvalidEncoding)?;
            return Ok(Self {
                target: ConnectTarget::Local,
                mode: LaunchMode::Link(crate::link::Link::parse(&link)?),
            });
        }
        let mut socket = None;
        #[cfg(feature = "integration-test")]
        let mut mode = LaunchMode::Normal;
        #[cfg(not(feature = "integration-test"))]
        let mode = LaunchMode::Normal;
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--help" | "-h" | "--build-info") => {
                    return Ok(Self {
                        target: ConnectTarget::Local,
                        mode: if arg == "--build-info" {
                            LaunchMode::BuildInfo
                        } else {
                            LaunchMode::Help
                        },
                    });
                }
                Some("--socket") => {
                    let value = args
                        .next()
                        .filter(|value| {
                            !value.is_empty() && !value.as_encoded_bytes().starts_with(b"-")
                        })
                        .ok_or(CliError::MissingSocketPath)?;
                    if socket.replace(value).is_some() {
                        return Err(CliError::DuplicateSocket);
                    }
                }
                #[cfg(feature = "integration-test")]
                Some(flag @ ("--integration-test" | "--sidebar-test" | "--performance-test")) => {
                    let next = match flag {
                        "--integration-test" => LaunchMode::Integration,
                        "--sidebar-test" => LaunchMode::Sidebar,
                        _ => LaunchMode::Performance,
                    };
                    if mode != LaunchMode::Normal {
                        return Err(CliError::ConflictingTestModes);
                    }
                    mode = next;
                }
                _ => {
                    return Err(CliError::UnknownOption(arg));
                }
            }
        }
        #[cfg(feature = "integration-test")]
        {
            if matches!(mode, LaunchMode::Sidebar | LaunchMode::Performance) && socket.is_some() {
                return Err(CliError::ConflictingFixtureOptions);
            }
            if mode == LaunchMode::Integration && socket.is_none() {
                return Err(CliError::MissingIntegrationSocket);
            }
            #[cfg(not(any(target_os = "macos", windows)))]
            if mode == LaunchMode::Performance {
                return Err(CliError::UnsupportedPerformancePlatform);
            }
        }
        let target = socket.map_or(ConnectTarget::Local, |path| {
            ConnectTarget::Socket(path.into())
        });
        Ok(Self { target, mode })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
