// ABOUTME: Translates Pi RPC events and ACP requests without owning I/O.
// ABOUTME: Uses one opening owner to gate session identity and discovery failures.
//! The translation between the session's ACP and pi's RPC mode, free of
//! threads and pipes.

use std::collections::HashMap;

use acp::v1 as wire;
use serde_json::{Value, json};

use super::tools;
use crate::harness::{Out, Translator, out};
use crate::project::PromptUsage;
use crate::rpc::Inbound;

mod opening;
mod turn;

const MODEL: &str = "model";
const THINKING: &str = "thinking";
/// pi's thinking levels, for a model that reasons.
const LEVELS: [&str; 6] = ["off", "minimal", "low", "medium", "high", "xhigh"];
/// Models kept from pi's list, so a long list cannot grow the session.
const MAX_MODELS: usize = 64;

/// What a command sent to pi is waiting for.
#[derive(Debug)]
enum Pending {
    State,
    Models,
    Commands,
    Prompt(u64),
    Stats,
    /// A model or thinking change, answering the session's request.
    Setting {
        acp: wire::RequestId,
        setting: &'static str,
        value: String,
    },
    Abort,
}

/// A dialog an extension opened, waiting for its answer.
#[derive(Debug)]
struct Dialog {
    id: String,
    /// `select` answers with its option's text; `confirm` with yes or no.
    confirm: bool,
    options: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct Bridge {
    /// pi's commands, held until the session is open.
    commands: Option<Value>,
    dialogs: HashMap<i64, Dialog>,
    /// Calls running now, by id, with their tool's name.
    calls: HashMap<String, String>,
    /// The message pi is streaming, numbered as pi gives messages no id.
    message: u64,
    model: Option<String>,
    next_id: u64,
    next_request: i64,
    opening: opening::State,
    pending: HashMap<String, Pending>,
    thinking: Option<String>,
    turn: turn::Turn,
}

impl Translator for Bridge {
    fn start(&mut self) -> Vec<Out> {
        vec![
            self.command(json!({"type": "get_state"}), Pending::State),
            self.command(json!({"type": "get_available_models"}), Pending::Models),
            self.command(json!({"type": "get_commands"}), Pending::Commands),
        ]
    }

    fn on_session(&mut self, inbound: Inbound) -> Vec<Out> {
        match inbound {
            Inbound::Request { id, method, params } => self.request(id, &method, params),
            Inbound::Notification { method, .. } if method == "session/cancel" => {
                if !matches!(
                    self.turn.transition(turn::Event::Cancel),
                    turn::Effect::Abort
                ) {
                    return Vec::new();
                }
                vec![self.command(json!({"type": "abort"}), Pending::Abort)]
            }
            Inbound::Response { id, result } => {
                let wire::RequestId::Number(id) = id else {
                    return Vec::new();
                };
                let Some(dialog) = self.dialogs.remove(&id) else {
                    return Vec::new();
                };
                let chosen = out::selected(&result);
                let reply = match (dialog.confirm, chosen) {
                    (true, Some(option)) => json!({"confirmed": option == "yes"}),
                    (false, Some(option)) => match option
                        .strip_prefix("option-")
                        .and_then(|index| index.parse::<usize>().ok())
                        .and_then(|index| dialog.options.get(index))
                    {
                        Some(value) => json!({"value": value}),
                        None => json!({"cancelled": true}),
                    },
                    (_, None) => json!({"cancelled": true}),
                };
                let mut reply = reply;
                reply["type"] = json!("extension_ui_response");
                reply["id"] = json!(dialog.id);
                vec![Out::Harness(reply)]
            }
            Inbound::Notification { .. } | Inbound::Closed(_) => Vec::new(),
        }
    }

    fn on_harness(&mut self, message: &Value) -> Vec<Out> {
        match message.get("type").and_then(Value::as_str) {
            Some("response") => self.response(message),
            Some("extension_ui_request") => self.dialog(message),
            Some(kind) => self.event(kind, message),
            None => Vec::new(),
        }
    }
}

impl Bridge {
    fn command(&mut self, mut command: Value, pending: Pending) -> Out {
        self.next_id += 1;
        let id = format!("herdr-{}", self.next_id);
        command["id"] = json!(id);
        self.pending.insert(id, pending);
        Out::Harness(command)
    }

    fn request(&mut self, id: wire::RequestId, method: &str, params: Value) -> Vec<Out> {
        match method {
            "initialize" => vec![out::reply(id, out::initialized())],
            "session/new" | "session/load" => {
                // Pi accepts shortened loaded ids; keep the caller's identity.
                let loaded = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                self.transition_opening(opening::Event::Ask { acp: id, loaded })
            }
            "session/prompt" => self.prompt(id, params),
            "session/set_config_option" => {
                let Some((setting, value)) = out::chosen(params) else {
                    return vec![out::reply(id, Err(wire::Error::invalid_params()))];
                };
                match setting.as_str() {
                    MODEL => {
                        let Some((provider, model)) = value.split_once('/') else {
                            return vec![out::reply(id, Err(wire::Error::invalid_params()))];
                        };
                        let command =
                            json!({"type": "set_model", "provider": provider, "modelId": model});
                        vec![self.command(
                            command,
                            Pending::Setting {
                                acp: id,
                                setting: MODEL,
                                value,
                            },
                        )]
                    }
                    THINKING if LEVELS.contains(&value.as_str()) => {
                        let command = json!({"type": "set_thinking_level", "level": value});
                        vec![self.command(
                            command,
                            Pending::Setting {
                                acp: id,
                                setting: THINKING,
                                value,
                            },
                        )]
                    }
                    _ => vec![out::reply(id, Err(wire::Error::invalid_params()))],
                }
            }
            _ => vec![out::reply(id, Err(wire::Error::method_not_found()))],
        }
    }

    fn session(&self) -> &str {
        self.opening.session().unwrap_or_default()
    }

    fn transition_opening(&mut self, event: opening::Event) -> Vec<Out> {
        let mut outs = Vec::new();
        for effect in self.opening.transition(event) {
            match effect {
                opening::Effect::Failed(acp, message) => {
                    outs.push(out::reply(acp, Err(out::failure(message))))
                }
                opening::Effect::Settings(state) => {
                    self.model = model_id(state.get("model"));
                    self.thinking = state
                        .get("thinkingLevel")
                        .and_then(Value::as_str)
                        .map(str::to_owned);
                }
                opening::Effect::Opened(acp) => {
                    outs.push(out::reply(
                        acp,
                        out::opened(self.session(), None, &self.options()),
                    ));
                    if let Some(commands) = self.commands.take() {
                        outs.extend(self.commands(Some(&commands)));
                    }
                }
            }
        }
        outs
    }

    fn options(&self) -> Vec<wire::SessionConfigOption> {
        let mut options = Vec::new();
        let models = self.opening.models();
        // pi names its model "unknown" until a provider is set up.
        if let Some(model) = self.model.as_ref().filter(|_| !models.is_empty()) {
            options.push(out::select(
                MODEL,
                "Model",
                wire::SessionConfigOptionCategory::Model,
                model,
                models
                    .iter()
                    .map(|(id, name, _)| (id.clone(), name.clone(), None)),
            ));
        }
        let reasons = models
            .iter()
            .any(|(id, _, reasons)| Some(id) == self.model.as_ref() && *reasons);
        if let (true, Some(level)) = (reasons, &self.thinking) {
            options.push(out::select(
                THINKING,
                "Thinking",
                wire::SessionConfigOptionCategory::ThoughtLevel,
                level,
                LEVELS
                    .iter()
                    .map(|level| ((*level).to_owned(), capitalized(level), None)),
            ));
        }
        options
    }

    fn prompt(&mut self, id: wire::RequestId, params: Value) -> Vec<Out> {
        let Ok(request) = serde_json::from_value::<wire::PromptRequest>(params) else {
            return vec![out::reply(id, Err(wire::Error::invalid_params()))];
        };
        let request_id = self.next_id + 1;
        if let turn::Effect::Busy(id) = self.turn.transition(turn::Event::Start {
            prompt: id,
            request: request_id,
        }) {
            return vec![out::reply(id, Err(out::failure(out::BUSY)))];
        }
        let mut text = Vec::new();
        let mut images = Vec::new();
        for block in &request.prompt {
            match block {
                wire::ContentBlock::Text(content) => text.push(content.text.clone()),
                wire::ContentBlock::Image(image) => images.push(json!({
                    "type": "image", "data": image.data, "mimeType": image.mime_type,
                })),
                wire::ContentBlock::Resource(resource) => {
                    if let wire::EmbeddedResourceResource::TextResourceContents(file) =
                        &resource.resource
                    {
                        text.push(out::embedded_file(&file.uri, &file.text));
                    }
                }
                // pi reads a file named with `@`.
                wire::ContentBlock::ResourceLink(link) => {
                    text.push(format!("@{}", out::file_path(&link.uri)));
                }
                _ => {}
            }
        }
        let mut command = json!({"type": "prompt", "message": text.join("\n\n")});
        if !images.is_empty() {
            command["images"] = Value::Array(images);
        }
        vec![self.command(command, Pending::Prompt(request_id))]
    }

    fn response(&mut self, message: &Value) -> Vec<Out> {
        let Some(pending) = message
            .get("id")
            .and_then(Value::as_str)
            .and_then(|id| self.pending.remove(id))
        else {
            return Vec::new();
        };
        let data = message.get("data").cloned().unwrap_or(Value::Null);
        let failed = (message.get("success") != Some(&Value::Bool(true))).then(|| {
            message
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("pi refused the command.")
                .to_owned()
        });
        match pending {
            Pending::State => self.transition_opening(opening::Event::State(
                failed.map_or(Ok(data), |message| Err(out::failure(message))),
            )),
            Pending::Models => self.transition_opening(opening::Event::Models(
                failed.map_or(Ok(data), |message| Err(out::failure(message))),
            )),
            Pending::Commands => {
                let commands = data.get("commands").cloned().unwrap_or(Value::Null);
                if self.opening.session().is_some() {
                    self.commands(Some(&commands))
                } else {
                    self.commands = Some(commands);
                    Vec::new()
                }
            }
            Pending::Prompt(request) => {
                let Some(error) = failed else {
                    // Accepted: the turn ends with `agent_end`.
                    return Vec::new();
                };
                match self.turn.transition(turn::Event::Refused(request)) {
                    turn::Effect::Completed(id) => vec![out::reply(id, Err(out::failure(error)))],
                    _ => Vec::new(),
                }
            }
            Pending::Stats => self.stats(&data),
            Pending::Setting {
                acp,
                setting,
                value,
            } => {
                if let Some(error) = failed {
                    return vec![out::reply(acp, Err(out::failure(error)))];
                }
                if setting == MODEL {
                    self.model = model_id(Some(&data)).or(Some(value));
                } else {
                    self.thinking = Some(value);
                }
                vec![out::reply(
                    acp,
                    Ok(json!({"configOptions": self.options()})),
                )]
            }
            Pending::Abort => Vec::new(),
        }
    }

    fn commands(&self, commands: Option<&Value>) -> Vec<Out> {
        let commands = commands
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|command| {
                let text = |key: &str| command.get(key).and_then(Value::as_str);
                Some(wire::AvailableCommand::new(
                    text("name")?,
                    text("description").unwrap_or_default(),
                ))
            })
            .collect();
        vec![out::update(
            self.session(),
            wire::SessionUpdate::AvailableCommandsUpdate(wire::AvailableCommandsUpdate::new(
                commands,
            )),
        )]
    }

    fn event(&mut self, kind: &str, message: &Value) -> Vec<Out> {
        let session = self.session().to_owned();
        let text = |key: &str| message.get(key).and_then(Value::as_str);
        match kind {
            "message_start"
                if message.pointer("/message/role").and_then(Value::as_str)
                    == Some("assistant") =>
            {
                self.message += 1;
                Vec::new()
            }
            "message_update" => {
                let event = message.get("assistantMessageEvent").unwrap_or(&Value::Null);
                let thought = match event.get("type").and_then(Value::as_str) {
                    Some("text_delta") => false,
                    Some("thinking_delta") => true,
                    _ => return Vec::new(),
                };
                let Some(delta) = event
                    .get("delta")
                    .and_then(Value::as_str)
                    .filter(|delta| !delta.is_empty())
                else {
                    return Vec::new();
                };
                let id = format!("pi-{}", self.message);
                vec![out::chunk(&session, thought, delta, Some(&id))]
            }
            "tool_execution_start" => {
                let (Some(id), Some(name)) = (text("toolCallId"), text("toolName")) else {
                    return Vec::new();
                };
                self.calls.insert(id.to_owned(), name.to_owned());
                let args = message.get("args").unwrap_or(&Value::Null);
                vec![out::update(
                    &session,
                    wire::SessionUpdate::ToolCall(tools::call(id, name, args)),
                )]
            }
            "tool_execution_end" => {
                let Some(id) = text("toolCallId") else {
                    return Vec::new();
                };
                let name = self.calls.remove(id).unwrap_or_default();
                let failed = message.get("isError") == Some(&Value::Bool(true));
                let result = message.get("result").unwrap_or(&Value::Null);
                let fields = tools::finished(&name, result, failed);
                vec![out::update(
                    &session,
                    wire::SessionUpdate::ToolCallUpdate(wire::ToolCallUpdate::new(
                        id.to_owned(),
                        fields,
                    )),
                )]
            }
            "agent_end" => self.finished(message.get("messages")),
            "compaction_end"
                if message
                    .get("result")
                    .is_some_and(|result| !result.is_null()) =>
            {
                vec![out::notice(
                    &session,
                    wire::NoticeSeverity::Info,
                    "pi compacted the conversation.",
                )]
            }
            "auto_retry_start" => vec![out::notice(
                &session,
                wire::NoticeSeverity::Warning,
                format!(
                    "Retrying after an error: {}",
                    text("errorMessage").unwrap_or("unknown")
                ),
            )],
            "extension_error" => text("error")
                .map(|error| out::notice(&session, wire::NoticeSeverity::Error, error))
                .into_iter()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Ends the prompt from how pi's last answer stopped, and asks for the
    /// session's usage.
    fn finished(&mut self, messages: Option<&Value>) -> Vec<Out> {
        let turn::Effect::Completed(id) = self.turn.transition(turn::Event::Complete) else {
            return Vec::new();
        };
        self.calls.clear();
        let assistant: Vec<&Value> = messages
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|message| message.get("role").and_then(Value::as_str) == Some("assistant"))
            .collect();
        let last = assistant.last();
        let stop = last
            .and_then(|message| message.get("stopReason"))
            .and_then(Value::as_str);
        let result = match stop {
            Some("aborted") => Ok(wire::StopReason::Cancelled),
            Some("length") => Ok(wire::StopReason::MaxTokens),
            Some("error") => Err(last
                .and_then(|message| message.get("errorMessage"))
                .and_then(Value::as_str)
                .unwrap_or("pi stopped with an error.")
                .to_owned()),
            _ => Ok(wire::StopReason::EndTurn),
        };
        let total = |key: &str| -> u64 {
            assistant
                .iter()
                .filter_map(|message| {
                    message
                        .pointer(&format!("/usage/{key}"))
                        .and_then(Value::as_u64)
                })
                .sum()
        };
        let tokens = PromptUsage {
            cached_read_tokens: total("cacheRead"),
            input_tokens: total("input") + total("cacheWrite"),
            output_tokens: total("output"),
            // pi counts no reasoning tokens.
            thought_tokens: None,
            total_tokens: total("input")
                + total("cacheWrite")
                + total("cacheRead")
                + total("output"),
        };
        vec![
            out::turn_ended(id, result, Some(tokens)),
            self.command(json!({"type": "get_session_stats"}), Pending::Stats),
        ]
    }

    fn stats(&self, data: &Value) -> Vec<Out> {
        let Some(size) = data
            .pointer("/contextUsage/contextWindow")
            .and_then(Value::as_u64)
        else {
            return Vec::new();
        };
        let used = data
            .pointer("/contextUsage/tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let mut usage = wire::UsageUpdate::new(used, size);
        if let Some(cost) = data.get("cost").and_then(Value::as_f64) {
            usage = usage.cost(Some(wire::Cost::new(cost, "USD")));
        }
        vec![out::update(
            self.session(),
            wire::SessionUpdate::UsageUpdate(usage),
        )]
    }

    /// An extension's question as a permission request; its notes as notices.
    fn dialog(&mut self, message: &Value) -> Vec<Out> {
        let text = |key: &str| message.get(key).and_then(Value::as_str);
        let Some(id) = text("id") else {
            return Vec::new();
        };
        let title = text("title").unwrap_or("pi asks").to_owned();
        let (confirm, options) = match text("method") {
            Some("confirm") => (true, Vec::new()),
            Some("select") => (
                false,
                message
                    .get("options")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            ),
            Some("notify") => {
                return text("message")
                    .map(|note| out::notice(self.session(), wire::NoticeSeverity::Info, note))
                    .into_iter()
                    .collect();
            }
            // Free text has no place in a permission request.
            Some("input" | "editor") => {
                return vec![Out::Harness(
                    json!({"type": "extension_ui_response", "id": id, "cancelled": true}),
                )];
            }
            _ => return Vec::new(),
        };
        let offered: Vec<wire::PermissionOption> = if confirm {
            vec![
                out::manual(wire::PermissionOption::new(
                    "yes",
                    "Yes",
                    wire::PermissionOptionKind::AllowOnce,
                )),
                wire::PermissionOption::new("no", "No", wire::PermissionOptionKind::RejectOnce),
            ]
        } else {
            options
                .iter()
                .enumerate()
                .map(|(index, option)| {
                    out::manual(wire::PermissionOption::new(
                        format!("option-{index}"),
                        option.clone(),
                        wire::PermissionOptionKind::AllowOnce,
                    ))
                })
                .collect()
        };
        self.next_request += 1;
        self.dialogs.insert(
            self.next_request,
            Dialog {
                id: id.to_owned(),
                confirm,
                options,
            },
        );
        let fields = wire::ToolCallUpdateFields::new().title(Some(title));
        let request = wire::RequestPermissionRequest::new(
            self.session().to_owned(),
            wire::ToolCallUpdate::new(format!("pi-dialog-{id}"), fields),
            offered,
        );
        vec![out::ask(
            self.next_request,
            "session/request_permission",
            &request,
        )]
    }
}

/// A pi model's `provider/id`.
fn model_id(model: Option<&Value>) -> Option<String> {
    let model = model?;
    let provider = model.get("provider")?.as_str()?;
    let id = model.get("id")?.as_str()?;
    Some(format!("{provider}/{id}"))
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

#[cfg(test)]
mod tests;
