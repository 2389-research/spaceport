//! Embeds the `.ftl` files and writes the message table `fl!` is checked
//! against while compiling (see `locales.rs`).

mod locales;

/// The build script's own failures, as a typed error rather than the `&str`
/// Cargo accepts here.
#[derive(Debug)]
enum BuildError {
    MissingManifestDir,
    MissingOutDir,
    Locales(locales::Error),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifestDir => f.write_str("CARGO_MANIFEST_DIR is not set"),
            Self::MissingOutDir => f.write_str("OUT_DIR is not set"),
            Self::Locales(error) => write!(f, "invalid messages: {error}"),
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Locales(error) => Some(error),
            Self::MissingManifestDir | Self::MissingOutDir => None,
        }
    }
}

fn main() -> Result<(), BuildError> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=locales.rs");
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or(BuildError::MissingManifestDir)?,
    );
    let out =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(BuildError::MissingOutDir)?);
    locales::generate(&manifest, &out).map_err(BuildError::Locales)
}
