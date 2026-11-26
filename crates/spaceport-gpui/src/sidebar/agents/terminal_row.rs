// ABOUTME: Builds an agent's two sidebar lines from its terminal and endpoint.
// ABOUTME: Uses the same cleaned terminal title as mobile Home and the tab strip.
use spaceport_client::protocol::{ClientShellAgent, ClientShellSnapshot};
use spaceport_sidebar::tokens::{ResolvedToken, TextRole, TokenKind};

#[cfg(test)]
mod tests;

fn token(kind: TokenKind) -> ResolvedToken {
    ResolvedToken {
        kind,
        style: Default::default(),
    }
}

fn text(value: &str, role: TextRole) -> ResolvedToken {
    token(TokenKind::Text(value.to_owned().into(), role))
}

pub(in crate::sidebar) struct TerminalRow {
    tab_number: Option<String>,
    title: Option<String>,
    directory: Option<String>,
}

impl TerminalRow {
    pub(in crate::sidebar) fn from_snapshot(
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        pane_name: Option<&str>,
    ) -> Option<Self> {
        let pane = snapshot.panes.iter().find(|p| p.pane_id == agent.pane_id);
        let title = spaceport_truncate::agent_terminal_title(agent);
        // Keep the workspace row while a partial snapshot lacks terminal metadata.
        if pane.is_none() && title.is_none() {
            return None;
        }
        let tab_number = snapshot
            .tabs
            .iter()
            .find(|tab| tab.workspace_id == agent.workspace_id && tab.tab_id == agent.tab_id)
            .map(|tab| format!("#{}", tab.number));
        let directory = pane
            .and_then(|p| {
                [p.foreground_cwd.as_deref(), p.cwd.as_deref()]
                    .into_iter()
                    .flatten()
                    .find(|path| !path.trim().is_empty())
            })
            .map(|path| {
                let leaf = path
                    .trim_end_matches(['/', '\\'])
                    .rsplit(['/', '\\'])
                    .next()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(path);
                spaceport_truncate::one_line(leaf, 64)
            });
        Some(Self {
            tab_number,
            title: title.or_else(|| pane_name.map(str::to_owned)),
            directory,
        })
    }

    pub(in crate::sidebar) fn lines(
        &self,
        app: &str,
        host: &str,
        status: Option<&str>,
    ) -> Vec<Vec<ResolvedToken>> {
        let mut metadata = vec![token(TokenKind::StateIcon), text(app, TextRole::Agent)];
        if let Some(number) = &self.tab_number {
            metadata.push(text(number, TextRole::Secondary));
        }
        if let Some(dir) = &self.directory {
            metadata.push(text(dir, TextRole::Secondary));
        }
        metadata.push(text(host, TextRole::Secondary));
        if let Some(status) = status {
            metadata.push(text(status, TextRole::Status));
        }
        let title = self.title.as_deref().unwrap_or(app);
        vec![metadata, vec![text(title, TextRole::Workspace)]]
    }

    pub(in crate::sidebar) fn spoken(&self, app: &str, host: &str, state: &str) -> String {
        crate::sidebar::accessibility::agent_name(
            app,
            self.title.as_deref(),
            state,
            Some((
                self.tab_number.as_deref().unwrap_or_default(),
                self.directory.as_deref(),
            )),
            Some(host),
        )
    }
}
