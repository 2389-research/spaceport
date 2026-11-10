//! ABOUTME: Maps reported agent names to their executables.
//! ABOUTME: Supplies agent identities for explicit closed-agent reopen operations.

/// Agents Herdr's integrations report resumable sessions for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Claude,
    Codex,
    Opencode,
    Pi,
    Omp,
    Copilot,
    Devin,
    Droid,
    Kimi,
    Mastracode,
    Hermes,
    Qodercli,
    Qwen,
    Kilo,
    Cursor,
    Antigravity,
    Grok,
    Letta,
}

impl AgentKind {
    /// Every kind, in declaration order.
    pub const ALL: &[Self] = &[
        Self::Claude,
        Self::Codex,
        Self::Opencode,
        Self::Pi,
        Self::Omp,
        Self::Copilot,
        Self::Devin,
        Self::Droid,
        Self::Kimi,
        Self::Mastracode,
        Self::Hermes,
        Self::Qodercli,
        Self::Qwen,
        Self::Kilo,
        Self::Cursor,
        Self::Antigravity,
        Self::Grok,
        Self::Letta,
    ];

    /// The kind for the agent name Herdr reports.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "claude" => Self::Claude,
            "codex" => Self::Codex,
            "opencode" => Self::Opencode,
            "pi" => Self::Pi,
            "omp" => Self::Omp,
            "copilot" => Self::Copilot,
            "devin" => Self::Devin,
            "droid" => Self::Droid,
            "kimi" => Self::Kimi,
            "mastracode" => Self::Mastracode,
            "hermes" => Self::Hermes,
            "qodercli" => Self::Qodercli,
            "qwen" => Self::Qwen,
            "kilo" => Self::Kilo,
            "cursor" => Self::Cursor,
            "agy" => Self::Antigravity,
            "grok" => Self::Grok,
            "letta" => Self::Letta,
            _ => return None,
        })
    }

    /// The agent name Herdr reports.
    pub fn name(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Opencode => "opencode",
            Self::Pi => "pi",
            Self::Omp => "omp",
            Self::Copilot => "copilot",
            Self::Devin => "devin",
            Self::Droid => "droid",
            Self::Kimi => "kimi",
            Self::Mastracode => "mastracode",
            Self::Hermes => "hermes",
            Self::Qodercli => "qodercli",
            Self::Qwen => "qwen",
            Self::Kilo => "kilo",
            Self::Cursor => "cursor",
            Self::Antigravity => "agy",
            Self::Grok => "grok",
            Self::Letta => "letta",
        }
    }

    /// The program that runs this agent. Host script routes are POSIX, so
    /// Cursor's is never Windows' `cursor-agent.cmd`.
    pub fn binary(self) -> &'static str {
        match self {
            Self::Cursor => "cursor-agent",
            _ => self.name(),
        }
    }
}

#[cfg(test)]
mod tests;
