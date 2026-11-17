// ABOUTME: Owns Claude's prompt and requested interruption as one turn state.
// ABOUTME: Preserves Claude's cancellation result when a natural finish races it.
use acp::v1 as wire;

#[derive(Debug, Default)]
pub(super) enum Turn {
    #[default]
    Idle,
    Running {
        prompt: wire::RequestId,
    },
    Interrupting {
        prompt: wire::RequestId,
    },
}

pub(super) enum Event {
    Start(wire::RequestId),
    Cancel,
    Complete,
}

pub(super) enum Effect {
    None,
    Busy(wire::RequestId),
    Interrupt,
    Completed {
        prompt: wire::RequestId,
        cancelled: bool,
    },
}

impl Turn {
    /// The only writer of Claude's turn lifecycle.
    pub(super) fn transition(&mut self, event: Event) -> Effect {
        match event {
            Event::Start(prompt) => {
                if !matches!(self, Self::Idle) {
                    return Effect::Busy(prompt);
                }
                *self = Self::Running { prompt };
                Effect::None
            }
            Event::Cancel => {
                let Self::Running { prompt } = self else {
                    return Effect::None;
                };
                *self = Self::Interrupting {
                    prompt: prompt.clone(),
                };
                Effect::Interrupt
            }
            Event::Complete => match std::mem::take(self) {
                Self::Idle => Effect::None,
                Self::Running { prompt } => Effect::Completed {
                    prompt,
                    cancelled: false,
                },
                Self::Interrupting { prompt } => Effect::Completed {
                    prompt,
                    cancelled: true,
                },
            },
        }
    }
}
