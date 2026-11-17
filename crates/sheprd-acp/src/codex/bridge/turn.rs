// ABOUTME: Owns Codex's prompt, turn identity, and pending interruption together.
// ABOUTME: Fences delayed start replies and completions from other known turns.
use acp::v1 as wire;

#[derive(Debug, Default)]
pub(super) enum Turn {
    #[default]
    Idle,
    Starting {
        prompt: wire::RequestId,
        request: i64,
        cancel: bool,
    },
    Running {
        prompt: wire::RequestId,
        request: i64,
        id: String,
    },
    Interrupting {
        prompt: wire::RequestId,
        request: i64,
        id: String,
    },
}

pub(super) enum Event {
    Start {
        prompt: wire::RequestId,
        request: i64,
    },
    Cancel,
    Started {
        request: Option<i64>,
        id: String,
    },
    Refused(i64),
    Complete(Option<String>),
}

pub(super) enum Effect {
    None,
    Busy(wire::RequestId),
    Interrupt(String),
    Completed(wire::RequestId),
    Stale,
}

impl Turn {
    pub(super) fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// The only writer of Codex's turn lifecycle.
    pub(super) fn transition(&mut self, event: Event) -> Effect {
        match event {
            Event::Start { prompt, request } => {
                if !self.is_idle() {
                    return Effect::Busy(prompt);
                }
                *self = Self::Starting {
                    prompt,
                    request,
                    cancel: false,
                };
                Effect::None
            }
            Event::Cancel => match self {
                Self::Starting { cancel, .. } => {
                    *cancel = true;
                    Effect::None
                }
                Self::Running {
                    prompt,
                    request,
                    id,
                } => {
                    let (prompt, request, id) = (prompt.clone(), *request, id.clone());
                    *self = Self::Interrupting {
                        prompt,
                        request,
                        id: id.clone(),
                    };
                    Effect::Interrupt(id)
                }
                Self::Idle | Self::Interrupting { .. } => Effect::None,
            },
            Event::Started {
                request: source,
                id,
            } => {
                let Self::Starting {
                    prompt,
                    request,
                    cancel,
                } = self
                else {
                    return Effect::None;
                };
                if source.is_some_and(|source| source != *request) {
                    return Effect::Stale;
                }
                let (prompt, request, cancel) = (prompt.clone(), *request, *cancel);
                if cancel {
                    *self = Self::Interrupting {
                        prompt,
                        request,
                        id: id.clone(),
                    };
                    Effect::Interrupt(id)
                } else {
                    *self = Self::Running {
                        prompt,
                        request,
                        id,
                    };
                    Effect::None
                }
            }
            Event::Refused(refused) => match self {
                Self::Starting {
                    request, prompt, ..
                }
                | Self::Running {
                    request, prompt, ..
                }
                | Self::Interrupting {
                    request, prompt, ..
                } if *request == refused => {
                    let prompt = prompt.clone();
                    *self = Self::Idle;
                    Effect::Completed(prompt)
                }
                _ => Effect::Stale,
            },
            Event::Complete(completed) => {
                if let Self::Running { id, .. } | Self::Interrupting { id, .. } = self
                    && completed.as_ref().is_some_and(|completed| completed != id)
                {
                    return Effect::Stale;
                }
                // Completion may precede turn/start's reply or turn/started.
                match std::mem::take(self) {
                    Self::Idle => Effect::None,
                    Self::Starting { prompt, .. }
                    | Self::Running { prompt, .. }
                    | Self::Interrupting { prompt, .. } => Effect::Completed(prompt),
                }
            }
        }
    }
}
