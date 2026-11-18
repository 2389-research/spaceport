// ABOUTME: Owns Pi state/model discovery and ACP opening transitions.
// ABOUTME: Retains failures until requested and forbids empty session identities.
use acp::v1 as wire;
use serde_json::Value;

use super::model_id;
use crate::harness::out;

type Models = Vec<(String, String, bool)>;

#[derive(Debug)]
enum Handshake {
    // keep-sorted start
    Failed(String),
    Ready {
        state: Value,
        models: Models,
    },
    Waiting {
        state: Option<Value>,
        models: Option<Models>,
    },
    // keep-sorted end
}

#[derive(Debug, Default)]
enum Opening {
    // keep-sorted start
    Asked {
        acp: wire::RequestId,
        loaded: Option<String>,
    },
    #[default]
    None,
    Open(String),
    // keep-sorted end
}

pub(super) enum Event {
    // keep-sorted start
    Ask {
        acp: wire::RequestId,
        loaded: Option<String>,
    },
    Models(Result<Value, wire::Error>),
    State(Result<Value, wire::Error>),
    // keep-sorted end
}

pub(super) enum Effect {
    // keep-sorted start
    Failed(wire::RequestId, String),
    Opened(wire::RequestId),
    Settings(Value),
    // keep-sorted end
}

#[derive(Debug)]
pub(super) struct State {
    handshake: Handshake,
    opening: Opening,
}

impl Default for State {
    fn default() -> Self {
        Self {
            handshake: Handshake::Waiting {
                state: None,
                models: None,
            },
            opening: Opening::None,
        }
    }
}

impl State {
    pub(super) fn session(&self) -> Option<&str> {
        match &self.opening {
            Opening::Open(id) => Some(id),
            _ => None,
        }
    }

    pub(super) fn models(&self) -> &[(String, String, bool)] {
        match &self.handshake {
            Handshake::Ready { models, .. } => models,
            _ => &[],
        }
    }

    /// The sole writer of discovery state and the session's opening identity.
    pub(super) fn transition(&mut self, event: Event) -> Vec<Effect> {
        let mut effects = Vec::new();
        match event {
            Event::Ask { acp, loaded } => {
                if matches!(self.opening, Opening::Asked { .. }) {
                    return vec![Effect::Failed(
                        acp,
                        "A session is already opening.".to_owned(),
                    )];
                }
                let loaded = loaded.or_else(|| self.session().map(str::to_owned));
                self.opening = Opening::Asked { acp, loaded };
            }
            Event::State(Err(message)) | Event::Models(Err(message)) => {
                if matches!(self.handshake, Handshake::Waiting { .. }) {
                    self.handshake = Handshake::Failed(message.message.to_string());
                }
            }
            Event::State(Ok(data)) => {
                if let Handshake::Waiting { state, .. } = &mut self.handshake {
                    effects.push(Effect::Settings(data.clone()));
                    *state = Some(data);
                }
            }
            Event::Models(Ok(data)) => {
                if let Handshake::Waiting { models, .. } = &mut self.handshake {
                    *models = Some(
                        data.get("models")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|model| {
                                let id = model_id(Some(model))?;
                                let name = model
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .map_or_else(|| id.clone(), str::to_owned);
                                Some((id, name, model.get("reasoning") == Some(&Value::Bool(true))))
                            })
                            .take(out::MAX_MODELS)
                            .collect(),
                    );
                }
            }
        }
        if let Handshake::Waiting {
            state: Some(state),
            models: Some(models),
        } = &self.handshake
        {
            self.handshake = Handshake::Ready {
                state: state.clone(),
                models: models.clone(),
            };
        }
        let Opening::Asked { loaded, .. } = &self.opening else {
            return effects;
        };
        let result = match &self.handshake {
            Handshake::Waiting { .. } => return effects,
            Handshake::Failed(message) => Err(out::failure(message.clone())),
            Handshake::Ready { state, .. } => loaded
                .as_deref()
                .or_else(|| state.get("sessionId").and_then(Value::as_str))
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| out::failure("pi returned no session id.")),
        };
        let Opening::Asked { acp, .. } = std::mem::take(&mut self.opening) else {
            return effects;
        };
        match result {
            Ok(session) => {
                self.opening = Opening::Open(session);
                effects.push(Effect::Opened(acp));
            }
            Err(message) => effects.push(Effect::Failed(acp, message.message.to_string())),
        }
        effects
    }
}
