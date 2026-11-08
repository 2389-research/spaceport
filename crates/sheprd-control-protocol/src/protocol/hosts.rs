//! Hosts and what the app gathers about them: listening and forwarded ports,
//! plan usage, and load.
use serde::{Deserialize, Serialize};

/// How long `ports open` may wait for a new forward to listen.
pub const FORWARD_WAIT_SECONDS: u64 = 20;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostOp {
    /// The window's hosts and their connection status.
    List,
    /// Show `id` in the window, as choosing it in the sidebar does.
    Select {
        id: String,
    },
    /// Mute or unmute `host` (an endpoint ID or label; absent, the caller's
    /// host) as a whole.
    Mute {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        host: Option<String>,
        muted: bool,
    },
    /// Replaces the tags that group `id` in the sidebar.
    Tag {
        id: String,
        tags: Vec<String>,
    },
    /// Ports the caller's workspace's processes listen on, and forwards.
    Ports,
    /// Forward a port of the selected SSH host to this machine.
    Forward {
        port: u16,
    },
    Unforward {
        port: u16,
    },
    /// Show `localhost:port` of the caller's workspace in a browser tab,
    /// forwarding or tunnelling a remote port first.
    OpenPort {
        port: u16,
    },
    Usage,
    System,
}

impl HostOp {
    pub fn wait_seconds(&self) -> u64 {
        match self {
            Self::OpenPort { .. } => FORWARD_WAIT_SECONDS,
            _ => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub bind: String,
    pub process: Option<String>,
    /// Where this machine reaches it, once forwarded.
    pub local_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardInfo {
    pub host: String,
    pub port: u16,
    /// "starting", "listening", or "ended".
    pub state: String,
    pub local_url: Option<String>,
    /// Why it ended.
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsageInfo {
    pub provider: String,
    pub account: Option<String>,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f32,
    pub resets_in_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoadInfo {
    pub host: String,
    pub cpu_percent: Option<f32>,
    pub memory_used: Option<u64>,
    pub memory_total: Option<u64>,
    pub cores: Option<u32>,
    pub load: Option<[f32; 3]>,
    pub uptime_seconds: Option<u64>,
    pub disk_available: Option<u64>,
    pub disk_total: Option<u64>,
    /// Graphics cards that report their usage; absent on hosts without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_count: Option<u32>,
    /// The cards' mean busy share.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_percent: Option<f32>,
    /// Dedicated video memory, summed across cards; absent on Apple GPUs,
    /// which share system memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_memory_used: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_memory_total: Option<u64>,
}

/// A workspace's listening ports, and its host's forwards. Boxed in
/// [`Reply`], which every response carries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ports {
    pub host: String,
    pub workspace: String,
    pub listening: Vec<PortInfo>,
    pub forwards: Vec<ForwardInfo>,
    /// Why the listing may be empty, such as scanning being turned off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reply {
    Hosts {
        hosts: Vec<super::app::HostInfo>,
    },
    Ports(Box<Ports>),
    Usage {
        host: String,
        usage: Vec<UsageInfo>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
    System {
        load: Option<Box<LoadInfo>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        note: Option<String>,
    },
}

#[cfg(test)]
mod tests;
