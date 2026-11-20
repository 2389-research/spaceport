// ABOUTME: Defines the private native profile observation handshake and report schema.
// ABOUTME: Distinguishes actually presented frames from latest projected state without terminal content dumps.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

pub(crate) const MARKER: &str = "SP_PROFILE_READY";
pub(crate) const OBSERVER_ENV: &str = "SPACEPORT_WORKING_PROFILE_DIR";
#[cfg(unix)]
pub(crate) const SOCKET: &str = "profile.sock";
pub(crate) const READY: &str = "profile.ready";
pub(crate) const MAX_LINE: u64 = 65_536;

pub(crate) fn encode_reply(reply: &Reply) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(reply)?;
    ensure!(
        (bytes.len() as u64) < MAX_LINE,
        "profile observation exceeds bound"
    );
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn endpoint(directory: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    #[cfg(unix)]
    return Ok(directory.join(SOCKET));
    #[cfg(windows)]
    {
        let name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| {
                !name.is_empty()
                    && name.len() <= 100
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            })
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid profile directory name",
                )
            })?;
        let sid = sheprd_client::private_endpoint::user_sid()?;
        Ok(std::path::PathBuf::from(format!(
            r"\\.\pipe\spaceport-profile-{sid}-{name}"
        )))
    }
}

pub(crate) fn marker_visible(frame: &sheprd_client::protocol::FrameData) -> bool {
    frame.width > 0
        && frame.cells.chunks(usize::from(frame.width)).any(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
                .contains(MARKER)
        })
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Request {
    Observe { id: u64, capture: bool },
    Stop,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub(crate) enum Reply {
    Observation(Observation),
    Stopped,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Observation {
    pub(crate) request_id: u64,
    pub(crate) gui_pid: u32,
    pub(crate) release: bool,
    pub(crate) window_id: u64,
    pub(crate) window_active: bool,
    pub(crate) window_visible: bool,
    pub(crate) native_key: Option<bool>,
    pub(crate) app_active: Option<bool>,
    pub(crate) app_hidden: Option<bool>,
    pub(crate) native_activation: String,
    pub(crate) windows_hwnd: Option<i64>,
    pub(crate) windows_native: Option<WindowsNative>,
    pub(crate) workspace: Option<String>,
    pub(crate) synthetic_working_panes: Vec<String>,
    pub(crate) groups: Vec<Group>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct WindowsNative {
    pub(crate) hwnd: i64,
    pub(crate) gui_pid: u32,
    pub(crate) thread_id: u32,
    pub(crate) foreground_hwnd: i64,
    pub(crate) visible: bool,
    pub(crate) minimized: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Group {
    pub(crate) index: usize,
    pub(crate) primary: bool,
    pub(crate) shown_terminal: bool,
    pub(crate) connected: bool,
    pub(crate) bounds: [f32; 4],
    pub(crate) tab: Option<String>,
    pub(crate) projected_revision: Option<u64>,
    pub(crate) presented: Option<Frame>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Frame {
    pub(crate) boot: String,
    pub(crate) projection_revision: u64,
    pub(crate) surface_revision: u64,
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) pane_ids: Vec<String>,
    pub(crate) nonblank_cells: usize,
    pub(crate) paired_image_assets: usize,
    pub(crate) marker_visible: bool,
    pub(crate) current: bool,
}
