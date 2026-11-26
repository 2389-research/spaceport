//! The daemon's configured sidebar tokens, under the path the app's layouts
//! have always named them by. Reading the config's tokens into the rows a
//! layout paints lives in `spaceport-sidebar`; the app words each status,
//! which is the one thing here that needs the catalog.

pub(super) use spaceport_sidebar::tokens::{ResolvedToken, SpaceContext, agent_rows, space_rows};
pub(super) use spaceport_sidebar::tokens::{TextRole, TokenKind};

/// An Idle heading supplies the status word. Keep the configured token's style for its age.
pub(super) fn replace_idle_status(lines: &mut Vec<Vec<ResolvedToken>>, age: Option<&str>) {
    for line in lines.iter_mut() {
        line.retain_mut(|token| match &mut token.kind {
            TokenKind::Text(text, TextRole::Status) => {
                if let Some(age) = age {
                    *text = age.to_owned().into();
                }
                age.is_some()
            }
            _ => true,
        });
    }
    lines.retain(|line| !line.is_empty());
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
