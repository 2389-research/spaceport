// ABOUTME: Defines single-turn Ask Spaceport interactions without view or process ownership.
// ABOUTME: Keeps New Chat confirmation separate from request execution.

#[derive(Default)]
pub(super) struct Interaction {
    pub(super) confirm_new: bool,
}

impl Interaction {
    pub(super) fn new_chat(&mut self, working: bool) -> bool {
        self.confirm_new = working;
        !working
    }
}

#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;
