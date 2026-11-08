use std::fmt;

macro_rules! methods {
    ($($variant:ident => $name:literal, $required:literal;)+) => {
        /// The herdr-core methods this client calls.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Method {
            $($variant,)+
        }

        impl Method {
            pub const ALL: &[Method] = &[$(Method::$variant,)+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Method::$variant => $name,)+
                }
            }

            /// Whether a core without it cannot back the app.
            pub fn required(self) -> bool {
                match self {
                    $(Method::$variant => $required,)+
                }
            }
        }
    };
}

methods! {
    // keep-sorted start
    DocGet => "doc.get", true;
    DocSet => "doc.set", true;
    EventsSubscribe => "events.subscribe", true;
    Hello => "hello", false;
    MachineAdd => "machine.add", false;
    MachineConnect => "machine.connect", false;
    MachineList => "machine.list", false;
    MachineRename => "machine.rename", false;
    MachineRm => "machine.rm", false;
    TerminalAttach => "terminal.attach", true;
    TerminalGet => "terminal.get", false;
    TerminalKill => "terminal.kill", true;
    TerminalList => "terminal.list", true;
    TerminalRun => "terminal.run", true;
    // keep-sorted end
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
