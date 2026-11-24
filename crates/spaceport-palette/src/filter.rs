//! Which rows the palette lists: everything, or one kind of destination.

use crate::action::Action;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Navigation,
    Commands,
    Projects,
    /// Checkouts the user keeps a note on.
    Notes,
}

impl Filter {
    pub const ALL: [Self; 5] = [
        Self::All,
        Self::Navigation,
        Self::Commands,
        Self::Projects,
        Self::Notes,
    ];

    /// The English name its selectors use.
    pub fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Navigation => "navigation",
            Self::Commands => "commands",
            Self::Projects => "projects",
            Self::Notes => "notes",
        }
    }

    pub fn accepts(self, action: &Action) -> bool {
        self == Self::All
            || self
                == match action {
                    Action::Native(_) | Action::Configured(..) => Self::Commands,
                    Action::Go { .. } => Self::Navigation,
                    Action::Project(_) | Action::NewProject(_) => Self::Projects,
                    Action::Note { .. } => Self::Notes,
                }
    }
}
