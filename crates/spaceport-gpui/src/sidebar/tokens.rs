//! The daemon's configured sidebar tokens, under the path the app's layouts
//! have always named them by. Reading the config's tokens into the rows a
//! layout paints lives in `spaceport-sidebar`; the app words each status,
//! which is the one thing here that needs the catalog.

pub(super) use spaceport_sidebar::tokens::{ResolvedToken, SpaceContext, agent_rows, space_rows};
#[cfg(test)]
pub(super) use spaceport_sidebar::tokens::{TextRole, TokenKind};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
