use crate::terminal_attach::Error;
use spaceport_i18n::assertions::assert_localized;

#[test]
fn every_terminal_attach_variant_has_a_message() {
    assert_localized(&[
        Error::Busy,
        Error::Ended,
        Error::Io(std::io::Error::other("disk on fire")),
        Error::Protocol(crate::protocol::Error::FrameLimit),
        Error::Refused(None),
        Error::Refused(Some("another client holds the terminal".into())),
        Error::Rejected {
            version: 2,
            reason: "unsupported".into(),
        },
        Error::Scroll,
        Error::Size,
        Error::Unexpected,
    ]);
    assert_localized(&[crate::Error::TerminalAttach(Error::Ended)]);
}
