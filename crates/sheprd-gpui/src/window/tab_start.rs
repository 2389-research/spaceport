//! Shared tab response parsing and pane input for chat and closed-agent reopen.
//! The connection owns the single script reply slot across both launch flows.

use crate::SheprdWindow;
use serde_json::Value;
use sheprd_client::protocol::ClientPaneInputEvent;

/// The tab and root pane a successful `tab.create` response made.
/// A malformed success is absent; a daemon refusal keeps its structured error.
pub(crate) fn created_tab(response: &Value) -> crate::Result<Option<(String, String)>> {
    let result = match sheprd_client::response::tagged(response, "tab_created") {
        Ok(result) => result,
        Err(error @ sheprd_client::Error::Endpoint { .. }) => return Err(error.into()),
        Err(_) => return Ok(None),
    };
    let id = |value: &Value| {
        value
            .as_str()
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
    };
    Ok(id(&result["tab"]["tab_id"]).zip(id(&result["root_pane"]["pane_id"])))
}

impl SheprdWindow {
    /// Only one chat or closed-agent tab may use the connection's script reply slot.
    pub(crate) fn script_slot_busy(&self) -> bool {
        self.overview_reopening() || self.chat_starting()
    }

    /// Type a fixed launch line into the pane whose tab the daemon just made.
    pub(crate) fn type_script_line(&self, boot: &str, pane: &str, line: &str) -> crate::Result<()> {
        let handle = self.endpoints[self.selected_endpoint]
            .connection
            .handle
            .as_ref()
            .ok_or(crate::Error::NotConnected)?;
        Ok(handle.send_input(
            boot,
            pane,
            [
                ClientPaneInputEvent::TextCommit(line.into()),
                crate::menu::enter_key(),
            ],
        )?)
    }
}

#[cfg(test)]
mod tests;
