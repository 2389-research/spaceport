// Build-time tools (git, gh) run under Cargo, not beside the app: no window to avoid.
#![allow(clippy::disallowed_methods)]

mod build_identity;
mod locales;
mod preview_identity;
mod release_version;
mod windows_resource;

/// The build script's own failures, as a typed error rather than the `&str`
/// Cargo accepts here. Hand-written rather than derived: `tests/build_identity`
/// recompiles this script in an isolated offline crate, so it must keep
/// building with no dependencies of its own.
#[derive(Debug)]
enum BuildError {
    MissingManifestDir,
    MissingOutDir,
    InvalidPreview,
    Locales(locales::Error),
    InvalidPrNumber,
    PrNumberEncoding(std::env::VarError),
    WindowsResource(std::io::Error),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifestDir => f.write_str("CARGO_MANIFEST_DIR is not set"),
            Self::InvalidPreview => f.write_str("SPACEPORT_MACOS_PREVIEW_VERSION requires macOS and YYYYMMDD.COUNTER (no leading zeros)"),
            Self::MissingOutDir => f.write_str("OUT_DIR is not set"),
            Self::Locales(error) => write!(f, "invalid messages: {error}"),
            Self::InvalidPrNumber => {
                f.write_str("HERDR_BUILD_PR_NUMBER must be empty or a positive decimal integer")
            }
            Self::PrNumberEncoding(_) => f.write_str("HERDR_BUILD_PR_NUMBER is not valid Unicode"),
            Self::WindowsResource(_) => {
                f.write_str("could not embed the Windows executable resources")
            }
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PrNumberEncoding(error) => Some(error),
            Self::WindowsResource(error) => Some(error),
            Self::Locales(error) => Some(error),
            Self::MissingManifestDir
            | Self::MissingOutDir
            | Self::InvalidPrNumber
            | Self::InvalidPreview => None,
        }
    }
}

fn main() -> Result<(), BuildError> {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && std::env::var_os("CARGO_FEATURE_MACOS_UPDATER").is_some()
    {
        // Sparkle's install name uses @rpath inside the shipped app bundle.
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    }
    for file in [
        "preview_identity.rs",
        "development-app-id",
        "macos-preview-app-id",
    ] {
        println!("cargo:rerun-if-changed={file}");
    }
    println!("cargo:rerun-if-env-changed=SPACEPORT_MACOS_PREVIEW_VERSION");
    let preview = std::env::var("SPACEPORT_MACOS_PREVIEW_VERSION");
    let preview = match &preview {
        Ok(value) => Some(value.as_str()),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => return Err(BuildError::InvalidPreview),
    };
    let (app_id, version) = preview_identity::select(
        preview,
        &std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default(),
    )
    .ok_or(BuildError::InvalidPreview)?;
    println!("cargo:rustc-env=SPACEPORT_APP_ID={app_id}");
    println!("cargo:rustc-env=SPACEPORT_APP_VERSION={version}");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    println!("cargo:rustc-env=SPACEPORT_BUILD_PROFILE={profile}");
    println!("cargo:rerun-if-changed=build_identity.rs");
    println!("cargo:rerun-if-changed=locales.rs");
    println!("cargo:rerun-if-changed=release_version.rs");
    println!("cargo:rerun-if-changed=windows_resource.rs");
    println!("cargo:rerun-if-env-changed=HERDR_BUILD_PR_NUMBER");
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or(BuildError::MissingManifestDir)?,
    );
    let out =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(BuildError::MissingOutDir)?);
    locales::generate(&manifest, &out).map_err(BuildError::Locales)?;
    let identity = build_identity::detect(&manifest);
    windows_resource::embed(&manifest, identity.worktree).map_err(BuildError::WindowsResource)?;
    for path in &identity.watched {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let pr = match std::env::var("HERDR_BUILD_PR_NUMBER") {
        Ok(value) => build_identity::validate_pr(&value)
            .ok_or(BuildError::InvalidPrNumber)?
            .to_owned(),
        Err(std::env::VarError::NotPresent) if identity.worktree && !identity.detached => {
            build_identity::lookup_pr(
                &manifest,
                &identity.branch,
                std::path::Path::new("gh"),
                std::time::Duration::from_secs(2),
            )
            .unwrap_or_default()
        }
        Err(std::env::VarError::NotPresent) => String::new(),
        Err(error) => return Err(BuildError::PrNumberEncoding(error)),
    };
    println!(
        "cargo:rustc-env=HERDR_BUILD_WORKTREE={}",
        u8::from(identity.worktree)
    );
    println!("cargo:rustc-env=HERDR_BUILD_BRANCH={}", identity.branch);
    println!("cargo:rustc-env=HERDR_BUILD_PR={pr}");
    println!("cargo:rustc-env=HERDR_BUILD_COMMIT={}", identity.commit);
    Ok(())
}
