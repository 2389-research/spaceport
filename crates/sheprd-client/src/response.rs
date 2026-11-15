//! The envelope every Herdr API response arrives in: `result` when the
//! request ran, `error` when the daemon refused it. Only those two keys and
//! a result's `type` tag are fixed here; what a result or an unknown error
//! code holds stays its method's business.

use crate::{Error, Result, text::safe_text};
use serde::Deserialize;
use serde_json::Value;

/// How many characters of a daemon's error message are kept for display.
pub const MAX_MESSAGE_CHARS: usize = 1024;
const MAX_CODE_CHARS: usize = 128;
/// The code of an error whose body names none.
const UNNAMED_CODE: &str = "endpoint_error";

macro_rules! endpoint_error_codes {
    ($($(#[$doc:meta])* $variant:ident => $wire:literal,)*) => {
        /// An endpoint error code this client acts on. Codes are open-ended
        /// on the wire, so anything else is kept verbatim for display.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum EndpointErrorCode {
            $($(#[$doc])* $variant,)*
            Other(String),
        }

        impl EndpointErrorCode {
            /// The code as the daemon spells it.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $wire,)*
                    Self::Other(code) => code,
                }
            }
        }

        impl From<String> for EndpointErrorCode {
            fn from(code: String) -> Self {
                match code.as_str() {
                    $($wire => Self::$variant,)*
                    _ => Self::Other(code),
                }
            }
        }
    };
}

endpoint_error_codes! {
    // keep-sorted start
    PaneNotFound => "pane_not_found",
    /// A plugin's link handler claimed a click and then failed.
    PluginLinkFailed => "plugin_link_failed",
    QueryTooLarge => "query_too_large",
    /// The pane's content changed after the request's revision was read.
    StaleContent => "stale_content",
    // keep-sorted end
}

/// The refusal an `error` body carries. Bodies are untrusted and open-ended:
/// a bare string or an object without `code` or `message` still keeps what
/// the daemon said, bounded and without control characters. The message is
/// empty only when the body holds no text at all.
fn refusal(error: &Value) -> Error {
    let field = |name: &str| error.get(name).and_then(Value::as_str);
    let code = field("code");
    let message = field("message")
        .or_else(|| error.as_str())
        .or(code)
        .unwrap_or_default();
    Error::Endpoint {
        code: safe_text(code.unwrap_or(UNNAMED_CODE), MAX_CODE_CHARS).into(),
        message: safe_text(message, MAX_MESSAGE_CHARS),
    }
}

/// The `result` of a response, or the refusal its `error` carried. A null
/// `error` is no error, and a null `result` is no result.
pub fn result(response: &Value) -> Result<&Value> {
    let Some(envelope) = response.as_object() else {
        return Err(Error::ResponseSchema(serde::de::Error::custom(
            "response is not an object",
        )));
    };
    if let Some(error) = envelope.get("error").filter(|error| !error.is_null()) {
        return Err(refusal(error));
    }
    envelope
        .get("result")
        .filter(|result| !result.is_null())
        .ok_or(Error::ResponseMissingResult)
}

/// The `result` of a response whose `type` tag is `kind`.
pub fn tagged<'a>(response: &'a Value, kind: &str) -> Result<&'a Value> {
    let result = result(response)?;
    if result.get("type").and_then(Value::as_str) != Some(kind) {
        return Err(Error::ResponseType);
    }
    Ok(result)
}

/// The `result` of a response tagged `kind`, decoded as `T`.
pub fn typed<'a, T: Deserialize<'a>>(response: &'a Value, kind: &str) -> Result<T> {
    T::deserialize(tagged(response, kind)?).map_err(Error::ResponseSchema)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
