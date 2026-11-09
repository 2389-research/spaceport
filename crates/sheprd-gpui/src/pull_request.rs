//! Repository input helpers shared by the remaining local Git probe and
//! workspace close preflight.

pub(crate) use sheprd_github::pull_request::{
    Input, Origin, repository_input, run, workspace_input,
};
