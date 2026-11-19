//! Config loader failures, keeping the parse cause the loader hides.

use super::Error;

impl From<sheprd_settings::herdr_settings::Error> for Error {
    fn from(error: sheprd_settings::herdr_settings::Error) -> Self {
        Self::Settings(sheprd_settings::Error::from(error))
    }
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
