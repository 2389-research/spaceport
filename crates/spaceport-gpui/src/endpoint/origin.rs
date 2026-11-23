//! What a device origin is called, as the tooltip over its mark in the
//! sidebar says it. The origins and the mark each one draws are
//! `spaceport_sidebar::origin`'s; the words stay with the app that owns the
//! catalog, as every other status word does.

use crate::i18n::fl;
use spaceport_sidebar::origin::Origin;

/// What the mark stands for, as its tooltip says it.
pub(crate) fn describe(origin: Origin) -> String {
    match origin {
        Origin::Local => fl!("sidebar-origin-local"),
        Origin::Ssh => fl!("sidebar-origin-ssh"),
        Origin::Tailscale => fl!("sidebar-origin-tailscale"),
        Origin::Wsl => fl!("sidebar-origin-wsl"),
    }
}

#[cfg(test)]
mod tests;
