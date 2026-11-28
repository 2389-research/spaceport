// Build-time tools (git, gh) run under Cargo, not beside the app: no window to avoid.
#![allow(clippy::disallowed_methods)]

mod build_identity;
mod preview_identity;
mod release_version;
mod windows_resource;

/// The build script's own failures, as a typed error rather than the `&str`
/// Cargo accepts here. Hand-written rather than derived: `tests/build_identity`
/// recompiles this script in an isolated offline crate, so it must keep
/// building with no dependencies of its own.
#[derive(Debug)]
enum BuildError {
    ConflictingChannels,
    InvalidPreview,
    InvalidPrNumber,
    InvalidProduction,
    MissingManifestDir,
    PrNumberEncoding(std::env::VarError),
    WindowsResource(std::io::Error),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingManifestDir => f.write_str("CARGO_MANIFEST_DIR is not set"),
            Self::ConflictingChannels => f.write_str(
                "SPACEPORT_MACOS_PREVIEW_VERSION and SPACEPORT_PRODUCTION_VERSION are mutually exclusive",
            ),
            Self::InvalidPreview => f.write_str("SPACEPORT_MACOS_PREVIEW_VERSION requires macOS and YYYYMMDD.COUNTER (no leading zeros)"),
            Self::InvalidProduction => f.write_str("SPACEPORT_PRODUCTION_VERSION requires macOS and YYYYMMDD.COUNTER (no leading zeros)"),
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
            Self::MissingManifestDir
            | Self::ConflictingChannels
            | Self::InvalidPrNumber
            | Self::InvalidPreview
            | Self::InvalidProduction => None,
        }
    }
}

/// A version a build was asked to carry, or `None` when its variable is absent.
fn requested_version(name: &str, invalid: BuildError) -> Result<Option<String>, BuildError> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(invalid),
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
    println!("cargo:rerun-if-env-changed=SPACEPORT_PRODUCTION_VERSION");
    let preview = requested_version(
        "SPACEPORT_MACOS_PREVIEW_VERSION",
        BuildError::InvalidPreview,
    )?;
    let production = requested_version(
        "SPACEPORT_PRODUCTION_VERSION",
        BuildError::InvalidProduction,
    )?;
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let identity = match (preview.as_deref(), production.as_deref()) {
        (Some(_), Some(_)) => return Err(BuildError::ConflictingChannels),
        (Some(version), None) => {
            preview_identity::select(preview_identity::Requested::MacosPreview(version), &target)
                .ok_or(BuildError::InvalidPreview)?
        }
        (None, Some(version)) => {
            preview_identity::select(preview_identity::Requested::Production(version), &target)
                .ok_or(BuildError::InvalidProduction)?
        }
        (None, None) => preview_identity::select(preview_identity::Requested::Development, &target)
            .ok_or(BuildError::InvalidPreview)?,
    };
    println!(
        "cargo:rustc-env=SPACEPORT_BUILD_CHANNEL={}",
        identity.channel
    );
    println!("cargo:rustc-env=SPACEPORT_APP_ID={}", identity.app_id);
    println!("cargo:rustc-env=SPACEPORT_APP_VERSION={}", identity.version);
    let profile = std::env::var("PROFILE").unwrap_or_default();
    println!("cargo:rustc-env=SPACEPORT_BUILD_PROFILE={profile}");
    println!("cargo:rerun-if-changed=build_identity.rs");
    println!("cargo:rerun-if-changed=release_version.rs");
    println!("cargo:rerun-if-changed=windows_resource.rs");
    println!("cargo:rerun-if-env-changed=HERDR_BUILD_PR_NUMBER");
    let manifest = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or(BuildError::MissingManifestDir)?,
    );
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
