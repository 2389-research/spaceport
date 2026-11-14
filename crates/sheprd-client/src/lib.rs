//! A local/SSH/WSL gen1 client. All transport I/O runs on a dedicated worker.
//! No reconnect/replay: commands carry the boot ID of the snapshot they act on.
//! Drain `Client::events` on a GUI background task, never block the UI thread.
#![doc = include_str!("../README.md")]
// Tests start real processes; production code builds them with `new_command`.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod app_name;
mod catalog;
pub mod clipboard;
mod compat;
mod connect;
#[cfg(unix)]
mod control_dir;
mod discovery;
mod environment;
mod error;
mod event;
mod forward;
mod frame;
mod handle;
mod host_theme;
mod limits;
mod local_network;
pub mod local_path;
pub mod login_env;
mod method;
mod options;
pub mod private_endpoint;
mod process;
mod queue;
mod remote_host;
pub mod response;
mod script;
pub mod scrollback;
mod session;
pub mod sessions;
pub mod settings_files;
pub mod ssh;
mod surface_images;
pub mod terminal_attach;
pub mod text;
mod transport;
mod upload;
pub mod workspaces;
pub mod wsl;

#[cfg(all(test, unix))]
#[path = "../../test-support/executable.rs"]
mod test_executable;
#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[allow(unsafe_code)]
#[path = "../../test-support/process.rs"]
mod test_process;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;

pub use crossbeam_channel::Receiver;
pub use sheprd_protocol as protocol;

pub use app_name::APP_NAME;
pub use catalog::{
    SavedHost, load_saved_host_selection, load_saved_hosts, store_saved_host_selection,
    valid_profile_id,
};
pub use clipboard::{ClipboardImageCancellation, ClipboardImageUpload};
pub use compat::{MIN_HERDR_VERSION, VersionMismatch};
pub use connect::{connect, connect_with_connector, connect_with_surface_active};
pub use discovery::{ConnectTarget, local_var, session_socket, set_login_config_home};
pub use environment::set_command_environment;
/// Error returned when queueing commands; also available as the crate's `Error`.
pub use error::Error as SendError;
pub use error::{Error, Result, StorageOperation};
pub use event::ClientEvent;
pub use forward::{ForwardEvent, PortForward, preferred_local_port};
pub use handle::{Client, ClientHandle};
pub use host_theme::HostTheme;
pub use local_network::{LocalNetworkAccess, local_network_access};
pub use method::Method;
pub use options::ConnectOptions;
#[cfg(windows)]
pub use process::CREATE_NO_WINDOW;
pub use process::new_command;
pub use remote_host::RemoteHost;
pub use script::{ScriptHost, ScriptLimits, run_script, run_script_command, shell_quote};
pub use sessions::{
    LocalSession, RemoteSession, SessionState, delete_local_session, delete_remote_session,
    list_local_sessions, list_remote_sessions, list_running_sessions_in,
};
pub use ssh::mux::Lane as SshLane;
pub use ssh::{
    Destination, FORWARD_READY, HostProbe, MAX_UNVERIFIED_HOSTS, SshChannel, SshFailure,
    forward_command, resolve_destination, script_command, set_unverified_hosts,
    shared_script_command,
};
pub use surface_images::{
    MAX_IMAGE_BYTES, MAX_IMAGE_SIDE, MAX_IMAGES, MAX_PLACEMENTS, SurfaceImage, SurfaceImages,
    valid_asset,
};
pub use terminal_attach::TerminalAttach;
pub use transport::Stream;
#[cfg(windows)]
pub use transport::pipe_available;
pub use upload::{remove_uploaded_files, upload_files};
pub use wsl::{
    WslHost, WslHosts, add_wsl_host, delete_distro_session, list_distro_sessions, list_distros,
    load_wsl_hosts, probe_distro, remove_wsl_host, store_wsl_selection, valid_distro, wsl_paths,
    wsl_shell_command,
};
