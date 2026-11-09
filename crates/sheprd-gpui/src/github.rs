//! GitHub sign-in as the app uses it. `herdr-github` owns the session, its
//! requests, and the credential store; the app keeps each account's avatar
//! (`avatars::ProfileAvatars`) and draws the menus.

pub(crate) use sheprd_github::{
    Access, Account, Auth, Installation, Note, Profile, Source, Store, Tokens, VERIFY_URL, forget,
    select_store, store_note,
};
