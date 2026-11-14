// ABOUTME: Parses the three bounded observation commands using opaque source targets.
// ABOUTME: Keeps observation requests free of paths, commands, and mutation options.
use super::{Args, CliError, unknown};
use sheprd_control_protocol::protocol::observation::{Bounds, MAX_OFFSET, Operation, Target};
pub(super) fn parse(args: &mut Args) -> Result<Operation, CliError> {
    let verb = args.verb().unwrap_or_else(|| "sessions".into());
    let offset = args.number("--offset", 0..=MAX_OFFSET)?.unwrap_or(0);
    let limit = args.number("--limit", 1..=usize::MAX)?;
    match verb.to_str() {
        Some("sessions") => {
            let status = args
                .text_option("--status")?
                .map(|status| match status.as_str() {
                    "idle" => Ok(sheprd_client::protocol::AgentStatus::Idle),
                    "working" => Ok(sheprd_client::protocol::AgentStatus::Working),
                    "blocked" => Ok(sheprd_client::protocol::AgentStatus::Blocked),
                    "done" => Ok(sheprd_client::protocol::AgentStatus::Done),
                    "unknown" => Ok(sheprd_client::protocol::AgentStatus::Unknown),
                    _ => Err(CliError::InvalidValue("--status")),
                })
                .transpose()?;
            Ok(Operation::Sessions {
                offset,
                limit: limit.unwrap_or(32),
                status,
            })
        }
        Some("conversation" | "terminal") => {
            let max_bytes = args
                .number("--max-bytes", 1..=usize::MAX)?
                .unwrap_or(16 * 1024);
            let target = Target(args.required_text("an observation target from sessions")?);
            target
                .validate()
                .map_err(|_| CliError::InvalidValue("TARGET"))?;
            let bounds = Bounds {
                offset,
                limit: limit.unwrap_or(80),
                max_bytes,
            };
            Ok(if verb == "conversation" {
                Operation::Conversation { target, bounds }
            } else {
                Operation::Terminal { target, bounds }
            })
        }
        _ => Err(unknown("observation", verb)),
    }
}
