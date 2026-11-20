// ABOUTME: Exercises the profile reply producer and consumer at the shared line boundary.
// ABOUTME: Counts the actual serialized bytes and required newline without fixed schema totals.
use super::report::{self, Reply};
use anyhow::{Result, ensure};
use serde_json::json;
use std::io::{BufRead as _, BufReader, Read};

pub(super) fn read_reply(reader: impl Read) -> Result<Reply> {
    let mut line = String::new();
    BufReader::new(reader)
        .take(report::MAX_LINE + 1)
        .read_line(&mut line)?;
    ensure!(
        line.len() as u64 <= report::MAX_LINE && line.ends_with('\n'),
        "invalid profile reply size"
    );
    Ok(serde_json::from_str(&line)?)
}

fn reply_with_payload_size(size: usize) -> Result<Reply> {
    let mut reply: Reply = serde_json::from_value(json!({"kind":"observation","data":{
        "request_id":1,"gui_pid":1,"release":true,"window_id":1,
        "reduce_motion":false,"mark_layer_render_calls":0,
        "window_active":true,"window_visible":true,"native_activation":"",
        "workspace":null,"synthetic_working_panes":[],"groups":[]
    }}))?;
    let base = serde_json::to_vec(&reply)?.len();
    ensure!(base <= size, "requested reply smaller than the schema");
    let Reply::Observation(observation) = &mut reply else {
        anyhow::bail!("missing fixture observation");
    };
    observation.native_activation = "x".repeat(size - base);
    assert_eq!(serde_json::to_vec(&reply)?.len(), size);
    Ok(reply)
}

#[test]
fn maximum_complete_reply_round_trips_through_the_real_consumer() -> Result<()> {
    let reply = reply_with_payload_size(report::MAX_LINE as usize - 1)?;
    let encoded = report::encode_reply(&reply)?;
    assert_eq!(encoded.len() as u64, report::MAX_LINE);
    assert_eq!(
        serde_json::to_value(read_reply(encoded.as_slice())?)?,
        serde_json::to_value(reply)?
    );
    Ok(())
}

#[test]
fn producer_refuses_a_payload_whose_newline_exceeds_the_consumer_bound() -> Result<()> {
    let reply = reply_with_payload_size(report::MAX_LINE as usize)?;
    let mut unchecked = serde_json::to_vec(&reply)?;
    unchecked.push(b'\n');
    assert!(read_reply(unchecked.as_slice()).is_err());
    assert!(
        report::encode_reply(&reply).is_err(),
        "producer accepted a reply its real consumer rejects"
    );
    Ok(())
}
