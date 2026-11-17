// ABOUTME: Owns Codex handshake and thread-opening transitions.
// ABOUTME: Retains early failures and publishes only validated thread identities.
use acp::v1 as wire;
use serde_json::Value;

#[derive(Debug, Default)]
enum Handshake {
    #[default]
    Waiting,
    Ready,
    Failed(String),
}

#[derive(Debug, Default)]
enum Opening {
    #[default]
    None,
    Asked(Request),
    Thread {
        acp: wire::RequestId,
        result: Option<Value>,
    },
    Open(String),
}

#[derive(Debug, Default)]
enum Models {
    #[default]
    NotAsked,
    Waiting,
    Ready(Value),
}

#[derive(Debug)]
pub(super) struct Request {
    pub(super) acp: wire::RequestId,
    pub(super) cwd: String,
    pub(super) resume: Option<String>,
}

pub(super) enum Event {
    Ask(Request),
    Initialize(Result<Value, wire::Error>),
    Models(Result<Value, wire::Error>),
    Thread(Result<Value, wire::Error>),
}

pub(super) enum Effect {
    Initialized,
    Models,
    Thread(Request),
    Opened {
        acp: wire::RequestId,
        thread: Value,
        models: Value,
    },
    Failed(wire::RequestId, String),
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
            _ => None,
        }
    }

    /// The sole writer of handshake, model-list, and opening state.
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
                self.opening = Opening::Asked(request);
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
                let Opening::Thread { acp, .. } = std::mem::take(&mut self.opening) else {
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
                            result: Some(thread),
                        };
                    }
                    Ok(_) => {
                        return vec![Effect::Failed(
                            acp,
                            "Codex returned no thread id.".to_owned(),
                        )];
                    }
                    Err(message) => return vec![Effect::Failed(acp, message.message.to_string())],
                }
            }
        }
        if matches!(self.opening, Opening::Asked(_)) {
            match &self.handshake {
                Handshake::Waiting => return effects,
                Handshake::Failed(message) => {
                    let Opening::Asked(request) = std::mem::take(&mut self.opening) else {
                        return effects;
                    };
                    effects.push(Effect::Failed(request.acp, message.clone()));
                }
                Handshake::Ready => {
                    let Opening::Asked(request) = std::mem::take(&mut self.opening) else {
                        return effects;
                    };
                    if matches!(self.models, Models::NotAsked) {
                        self.models = Models::Waiting;
                        effects.push(Effect::Models);
                    }
                    self.opening = Opening::Thread {
                        acp: request.acp.clone(),
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
