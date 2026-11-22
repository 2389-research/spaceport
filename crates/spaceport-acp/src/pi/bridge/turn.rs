// ABOUTME: Owns Pi's prompt and pending cancellation as one turn state.
// ABOUTME: Fences late command failures by their originating prompt.
use acp::v1 as wire;

#[derive(Debug, Default)]
pub(super) enum Turn {
    #[default]
    Idle,
    Running {
        prompt: wire::RequestId,
        request: u64,
    },
    Interrupting {
        prompt: wire::RequestId,
        request: u64,
    },
}

pub(super) enum Event {
    Start {
        prompt: wire::RequestId,
        request: u64,
    },
    Cancel,
    Refused(u64),
    Complete,
}

pub(super) enum Effect {
    None,
    Busy(wire::RequestId),
    Abort,
    Completed(wire::RequestId),
}

impl Turn {
    /// The only writer of Pi's turn lifecycle.
    pub(super) fn transition(&mut self, event: Event) -> Effect {
        match event {
            Event::Start { prompt, request } => {
                if !matches!(self, Self::Idle) {
                    return Effect::Busy(prompt);
                }
                *self = Self::Running { prompt, request };
                Effect::None
            }
            Event::Cancel => {
                let Self::Running { prompt, request } = self else {
                    return Effect::None;
                };
                *self = Self::Interrupting {
                    prompt: prompt.clone(),
                    request: *request,
                };
                Effect::Abort
            }
            Event::Refused(refused) => match self {
                Self::Running { prompt, request } | Self::Interrupting { prompt, request }
                    if *request == refused =>
                {
                    let prompt = prompt.clone();
                    *self = Self::Idle;
                    Effect::Completed(prompt)
                }
                _ => Effect::None,
            },
            // Pi supplies no turn identity on agent_end. Keep its stop reason authoritative.
            Event::Complete => match std::mem::take(self) {
                Self::Idle => Effect::None,
                Self::Running { prompt, .. } | Self::Interrupting { prompt, .. } => {
                    Effect::Completed(prompt)
                }
            },
        }
    }
}
