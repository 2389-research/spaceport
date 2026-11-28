// ABOUTME: Owns Codex handshake and thread-opening transitions.
// ABOUTME: Retains early failures and publishes only validated thread identities.
use acp::v1 as wire;
use serde_json::Value;

#[derive(Debug, Default)]
enum Handshake {
    // keep-sorted start
    Failed(String),
    Ready,
    #[default]
    Waiting,
    // keep-sorted end
}

/// Pending openings retain the active identity until a validated success replaces it.
#[derive(Debug, Default)]
enum Opening {
    // keep-sorted start
    Asked {
        active: Option<String>,
        request: Request,
    },
    #[default]
    None,
    Open(String),
    Thread {
        acp: wire::RequestId,
        active: Option<String>,
        result: Option<Value>,
    },
    // keep-sorted end
}

#[derive(Debug, Default)]
enum Models {
    // keep-sorted start
    #[default]
    NotAsked,
    Ready(Value),
    Waiting,
    // keep-sorted end
}

#[derive(Debug)]
pub(super) struct Request {
    pub(super) acp: wire::RequestId,
    pub(super) cwd: String,
    pub(super) resume: Option<String>,
}

pub(super) enum Event {
    // keep-sorted start
    Ask(Request),
    Initialize(Result<Value, wire::Error>),
    Models(Result<Value, wire::Error>),
    Thread(Result<Value, wire::Error>),
    // keep-sorted end
}

pub(super) enum Effect {
    // keep-sorted start
    Failed(wire::RequestId, String),
    Initialized,
    Models,
    Opened {
        acp: wire::RequestId,
        thread: Value,
        models: Value,
    },
    Thread(Request),
    // keep-sorted end
}

#[derive(Debug, Default)]
pub(super) struct State {
    handshake: Handshake,
    models: Models,
    opening: Opening,
}

impl State {
    pub(super) fn session(&self) -> Option<&str> {
        match &self.opening {
            Opening::Open(id) => Some(id),
            Opening::Asked { active, .. } | Opening::Thread { active, .. } => active.as_deref(),
            Opening::None => None,
        }
    }

    /// The sole writer of handshake, model-list, and opening state.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 111 lines")]
    pub(super) fn transition(&mut self, event: Event) -> Vec<Effect> {
        let mut effects = Vec::new();
        match event {
            Event::Ask(request) => {
                if matches!(self.opening, Opening::Asked { .. } | Opening::Thread { .. }) {
                    return vec![Effect::Failed(
                        request.acp,
                        "A session is already opening.".to_owned(),
                    )];
                }
                self.opening = Opening::Asked {
                    active: self.session().map(str::to_owned),
                    request,
                };
            }
            Event::Initialize(Ok(_)) => {
                self.handshake = Handshake::Ready;
                effects.push(Effect::Initialized);
            }
            Event::Initialize(Err(message)) => {
                self.handshake =
                    Handshake::Failed(format!("Codex's handshake failed: {}", message.message));
            }
            Event::Models(result) => {
                // Model discovery is optional; a refused list has no settings.
                self.models = Models::Ready(result.unwrap_or(Value::Null));
            }
            Event::Thread(result) => {
                if !matches!(self.opening, Opening::Thread { .. }) {
                    return effects;
                }
                let Opening::Thread { acp, active, .. } = std::mem::take(&mut self.opening) else {
                    return effects;
                };
                match result {
                    Ok(thread)
                        if thread
                            .pointer("/thread/id")
                            .and_then(Value::as_str)
                            .is_some_and(|id| !id.is_empty()) =>
                    {
                        self.opening = Opening::Thread {
                            acp,
                            active,
                            result: Some(thread),
                        };
                    }
                    Ok(_) => {
                        self.opening = active.map_or(Opening::None, Opening::Open);
                        return vec![Effect::Failed(
                            acp,
                            "Codex returned no thread id.".to_owned(),
                        )];
                    }
                    Err(message) => {
                        self.opening = active.map_or(Opening::None, Opening::Open);
                        return vec![Effect::Failed(acp, message.message.to_string())];
                    }
                }
            }
        }
        if matches!(self.opening, Opening::Asked { .. }) {
            match &self.handshake {
                Handshake::Waiting => return effects,
                Handshake::Failed(message) => {
                    let Opening::Asked { active, request } = std::mem::take(&mut self.opening)
                    else {
                        return effects;
                    };
                    self.opening = active.map_or(Opening::None, Opening::Open);
                    effects.push(Effect::Failed(request.acp, message.clone()));
                }
                Handshake::Ready => {
                    let Opening::Asked { active, request } = std::mem::take(&mut self.opening)
                    else {
                        return effects;
                    };
                    if matches!(self.models, Models::NotAsked) {
                        self.models = Models::Waiting;
                        effects.push(Effect::Models);
                    }
                    self.opening = Opening::Thread {
                        acp: request.acp.clone(),
                        active,
                        result: None,
                    };
                    effects.push(Effect::Thread(request));
                }
            }
        }
        if let (
            Opening::Thread {
                acp,
                result: Some(thread),
                ..
            },
            Models::Ready(models),
        ) = (&self.opening, &self.models)
        {
            let effect = Effect::Opened {
                acp: acp.clone(),
                thread: thread.clone(),
                models: models.clone(),
            };
            let id = thread
                .pointer("/thread/id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            self.opening = Opening::Open(id);
            effects.push(effect);
        }
        effects
    }
}
