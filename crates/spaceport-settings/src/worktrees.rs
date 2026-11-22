//! `[worktrees]`: the hue linked Git worktrees are marked in, and where.
//!
//! The daemon already reports, per workspace, whether its checkout is a linked
//! worktree, so nothing here reads Git. These are the settings the app's marks
//! are drawn from.
use crate::{Error, Result, parse_hex};
use serde::Deserialize;

/// Where a linked worktree's marks are drawn.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    /// The sidebar: guide lines in the layouts that draw no tree of their
    /// own, and the branch glyph on a worktree standing alone.
    Sidebar,
    /// The edge between the workspace's tab strip and its terminals.
    Terminal,
}

/// `[worktrees]`, resolved: an optional hue and the surfaces that show it.
/// By default only the sidebar does; the rule along the tab strip is opt-in.
/// `apply_to = []` turns every mark off, leaving the sidebar as Herdr draws
/// it without them: the flat layouts keep their tree, the rest none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Worktrees {
    /// A configured hue for the glyph and the rule, as `0xRRGGBB`. Unset,
    /// the theme's blue.
    pub tint: Option<u32>,
    pub sidebar: bool,
    pub terminal: bool,
}

impl Default for Worktrees {
    fn default() -> Self {
        Self {
            tint: None,
            sidebar: true,
            terminal: false,
        }
    }
}

impl Worktrees {
    /// The marks for `surfaces` in `tint`, or the theme's hue without one.
    pub fn new(tint: Option<u32>, surfaces: &[Surface]) -> Self {
        Self {
            tint,
            sidebar: surfaces.contains(&Surface::Sidebar),
            terminal: surfaces.contains(&Surface::Terminal),
        }
    }
}

/// The table as written. Unset keys keep the defaults: the theme's hue in
/// the sidebar, and no rule along the tab strip.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct WorktreeSettings {
    tint: Option<String>,
    apply_to: Option<Vec<Surface>>,
}

impl WorktreeSettings {
    pub fn resolve(&self) -> Result<Worktrees> {
        let tint = self
            .tint
            .as_deref()
            .map(|value| {
                parse_hex(value).ok_or_else(|| Error::InvalidWorktreeTint(value.to_owned()))
            })
            .transpose()?;
        Ok(match &self.apply_to {
            Some(surfaces) => Worktrees::new(tint, surfaces),
            None => Worktrees {
                tint,
                ..Worktrees::default()
            },
        })
    }
}
