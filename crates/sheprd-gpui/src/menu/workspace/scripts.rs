//! The workspace menu runs repository scripts in visible tabs.
//! The scripts themselves are `crate::worktree_scripts`.

use crate::SheprdWindow;
use gpui::*;

impl SheprdWindow {
    /// Runs one of the menu workspace's scripts in a new tab of its checkout,
    /// once the daemon has named the checkout and the file is trusted.
    pub(super) fn run_workspace_script(
        &mut self,
        kind: crate::worktree_scripts::ScriptKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let launch = self.menu.target.as_ref().map(|target| {
            let tree = target.worktree.as_ref();
            crate::worktree_scripts::Launch {
                kind,
                endpoint: self.menu.endpoint_target,
                endpoint_id: self.endpoints[self.selected_endpoint].id.clone(),
                boot: target.boot_id.clone(),
                workspace: target.id.clone(),
                repo: tree.map_or_else(|| target.label.clone(), |tree| tree.label.clone()),
                repo_key: tree.map(|tree| tree.key.clone()).unwrap_or_default(),
                checkout: None,
                requested: true,
            }
        });
        let started = match launch {
            Some(launch) if launch.repo_key.is_empty() => {
                Err(crate::worktree_scripts::Error::NotGit.into())
            }
            Some(_) if !self.menu_target_current() || !self.live.status.is_connected() => {
                Err(crate::Error::StaleConnection)
            }
            Some(launch) => self.start_worktree_script(launch, cx),
            None => Err(crate::Error::StaleWorkspace),
        };
        self.dismiss_menu(window, cx);
        if let Err(error) = started {
            self.show_flash(crate::window::Flash::warning(error.to_string()), cx);
        }
    }
}
