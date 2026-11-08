//! Pure file and symbol ranking for the existing local CLI search command.
//! The native Go to File and Symbol picker is excluded from Spaceport.
mod rank;

pub(crate) use rank::{Mode, Query, rank};
