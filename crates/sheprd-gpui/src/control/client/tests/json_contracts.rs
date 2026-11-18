// ABOUTME: Checks stable JSON error identities and collection envelopes.
// ABOUTME: Keeps empty list results consistent across tab, window and host commands.
use super::*;
use crate::control::protocol::{
    app::{Reply as AppReply, WindowOp},
    hosts::{HostOp, Reply as HostReply},
    tabs::Reply as TabReply,
};

#[test]
fn json_errors_keep_the_server_failure_identity() {
    for (failure, expected) in [
        (ErrorCode::WorkspaceNotFound, "workspace_not_found"),
        (ErrorCode::NoWindow, "no_window"),
        (ErrorCode::NotFound, "not_found"),
        (ErrorCode::InvalidRequest, "invalid_request"),
        (ErrorCode::Failed, "failed"),
    ] {
        let error = crate::Error::from(crate::control::Error::Rejected {
            code: failure,
            message: "fixture".into(),
        });
        assert_eq!(code(&error), expected);
    }
}

fn empty_list(request: &Request) -> crate::Result<Response> {
    Ok(match request {
        Request::Tab(_) => Response::Tab {
            reply: TabReply::Tabs { tabs: Vec::new() },
        },
        Request::Window(_) => Response::App {
            reply: AppReply::Windows {
                windows: Vec::new(),
            },
        },
        Request::Host(_) => Response::Host {
            reply: HostReply::Hosts { hosts: Vec::new() },
        },
        _ => panic!("not a list"),
    })
}

#[test]
fn empty_lists_use_named_envelopes_and_the_nothing_exit_status() {
    let context = Context {
        origin: origin::Origin::Shell,
        json: true,
        window: None,
        transport: empty_list,
    };
    for (key, result) in [
        ("tabs", tabs::list(&context)),
        ("windows", app::window(WindowOp::List, &context)),
        ("hosts", hosts::host(HostOp::List, &context)),
    ] {
        let output = result.unwrap();
        assert_eq!(output.status, exit::NOTHING, "{key}");
        assert_eq!(output.json, serde_json::json!({ key: [] }), "{key}");
    }
}

#[test]
fn local_catalogs_also_use_named_collection_envelopes() {
    use crate::cli::control::{LanguageCommand, SoundCommand};
    let context = Context {
        origin: origin::Origin::Shell,
        json: true,
        window: None,
        transport: empty_list,
    };
    for (key, output) in [
        ("commands", local::commands().unwrap()),
        (
            "languages",
            settings::language(LanguageCommand::List, &context).unwrap(),
        ),
        (
            "sounds",
            settings::sound(SoundCommand::List, &context).unwrap(),
        ),
    ] {
        assert!(
            output.json[key]
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "{key}: {}",
            output.json
        );
    }
}
