// ABOUTME: Registers Core terminal presentation and worker tests.
// ABOUTME: Shares the isolated scripted Core fixture across lifecycle tests.
mod chats;
mod focus;
mod keys;
mod pane_owner;
mod panes;
mod paste;
mod screen;
#[cfg(unix)]
pub(crate) mod scripted_core;
mod selection;
mod spawn_identity;
mod spawn_queue;
mod worker;
pub(crate) use worker::recording_link;
