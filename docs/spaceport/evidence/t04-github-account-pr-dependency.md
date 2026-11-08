# T04/RM-GITHUB: remaining account and PR graph

Read-only source map after the Security cut at `1ceb52de`. No further app
code, Core state, account, provider service or personal configuration changed.

- `window.rs::poll_github` still drives `menu/github.rs::poll_github_at` on
  every tick. That method probes GitHub CLI sign-in, polls default, host and
  extra accounts, reads avatar state and raises account notices. The titlebar
  starts sign-in on demand and renders the profile/avatar slot
  (`titlebar.rs`); `menu/chrome.rs` opens `render_github_auth` from
  `menu/github.rs`. `config.rs` and `config/github.rs` still parse/save
  `[github]` accounts. This is active product account behavior, not a shell.
- The PR/Issues Activity destinations remain in
  `sheprd-control-protocol/src/protocol/app.rs`, `sidebar/activity.rs`,
  `orchestrator/section.rs`, `orchestrator/host.rs` and
  `orchestrator/view.rs`. PR actions also run through `menu/pr_actions.rs`,
  `sheprd-github/src/pr_actions.rs`, `repo_items/` and the `sheprd-github` crate. Their
  repository worker and Core view ownership cross T03's reader work, so a
  standalone account/menu deletion would strand live PR and issue routes.
- `settings_window/cloud_devices/{azure,boat,coder,daytona,opensandbox,sprites}.rs`
  imports `github::Store`, but `sheprd-github/src/store.rs` re-exports the
  generic `sheprd_credentials::Store`. These provider settings use that
  policy for their own secrets; the import path does not make those secrets
  GitHub accounts. Daytona provider files and T03 Core reader files were
  outside this cut. The later account removal must move this generic import
  to its actual owner before deleting the GitHub facade.
- `sheprd-vcs` and `sheprd-orchestrator` still depend on `sheprd-github`
  (`Cargo.toml`), so the crate cannot go with the Security leaf. The
  updater has its own release HTTP transport and no `sheprd-github` Cargo
  dependency (`crates/sheprd-updater/Cargo.toml`); that release transport and
  the generic credentials crate must survive their product-specific cuts.

Next coherent cut: settle the retained T03 Core reader/view ownership and
provider credential import seam, then remove the PR/Issues and account
workers, actions, menus, settings, assets and dependency edges together with
negative startup/CLI/UI checks. Retain only the generic profile slot and
ordinary workspace/terminal/chat content. No dormant GitHub polling or
hidden disabled account state should remain. T04 stays blocked on G-LINK;
A29 and A30 remain `not_run`.
