// ABOUTME: Verifies bounded observation requests and their JSON control transport.
// ABOUTME: Rejects malformed targets and preserves unknown native agent status.
#![allow(clippy::unwrap_used)]
use sheprd_control_protocol::protocol::{Call, Caller, Request, Response, observation::*};

#[test]
fn requests_round_trip_with_only_three_operations() {
    for op in [
        Operation::Sessions {
            offset: 0,
            limit: 32,
            status: None,
        },
        Operation::Conversation {
            target: Target("test-target".into()),
            bounds: Bounds::default(),
        },
        Operation::Terminal {
            target: Target("test-target".into()),
            bounds: Bounds::default(),
        },
    ] {
        let request = Request::Observation(Call {
            caller: Caller::default(),
            op,
        });
        let wire = serde_json::to_vec(&request).unwrap();
        assert_eq!(serde_json::from_slice::<Request>(&wire).unwrap(), request);
    }
    assert!(
        serde_json::from_str::<Request>(
            r#"{"method":"observation","caller":{},"op":{"op":"shell","command":"echo x"}}"#
        )
        .is_err()
    );
}

#[test]
fn bounds_reject_zero_and_clamp_large_requests() {
    assert_eq!(
        Bounds {
            offset: 0,
            limit: 0,
            max_bytes: 1
        }
        .validated(),
        Err(Error::InvalidBounds)
    );
    let bounds = Bounds {
        offset: 0,
        limit: usize::MAX,
        max_bytes: usize::MAX,
    }
    .validated()
    .unwrap();
    assert_eq!(bounds.limit, MAX_EXCERPT_ITEMS);
    assert_eq!(bounds.max_bytes, MAX_EXCERPT_BYTES);
    assert_eq!(
        Target("/tmp/socket".into()).validate(),
        Err(Error::InvalidTarget)
    );
}

#[test]
fn unavailable_reply_round_trips_as_typed_result() {
    let response = Response::Observation {
        reply: Reply::Unavailable {
            target: Some(Target("test-target".into())),
            reason: UnavailableReason::StaleTarget,
        },
    };
    let wire = serde_json::to_vec(&response).unwrap();
    assert_eq!(serde_json::from_slice::<Response>(&wire).unwrap(), response);
}

#[test]
fn observation_validation_keeps_its_typed_root_error() {
    let error: sheprd_control_protocol::Error = Error::InvalidBounds.into();
    assert!(matches!(
        error,
        sheprd_control_protocol::Error::Observation(Error::InvalidBounds)
    ));
    assert!(error.to_string().contains("bounds"));
}
