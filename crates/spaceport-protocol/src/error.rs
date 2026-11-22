use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not talk to the daemon: {0}")]
    Io(#[from] io::Error),
    #[error("A message could not be written for the daemon: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("A message from the daemon could not be read: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("A message from the daemon is larger than this app accepts.")]
    FrameLimit,
    #[error("The daemon sent an empty message.")]
    EmptyFrame,
    #[error("A clipboard image must hold 1 byte through 16 MiB.")]
    ClipboardImageSize,
    #[error("A clipboard image is in a format this app cannot read.")]
    ClipboardImageExtension,
    #[error("A clipboard image's target must hold 1 through 1024 bytes.")]
    ClipboardImageTarget,
    #[error("The daemon sent extra bytes after a message.")]
    TrailingBytes,
    #[error("A pane screen arrived with the wrong number of cells for its size.")]
    CellCount,
    #[error("A pane screen named a link it does not carry.")]
    HyperlinkIndex,
    #[error("A pane screen put the cursor outside the screen.")]
    CursorBounds,
    #[error("A pane screen update named a pane or revision it does not match.")]
    PatchIdentity,
    #[error("A pane screen update named a row the screen does not have.")]
    PatchRowBounds,
    #[error("A pane screen update tried to change the pane's size.")]
    PatchGeometry,
    #[error("A pane screen update put the cursor outside the screen.")]
    PatchCursorBounds,
    #[error("An encoded pane screen is larger than a message may be.")]
    EncodingLimit,
    #[error("An encoded pane screen could not be decoded: {0}")]
    EncodingPayload(#[from] base64::DecodeError),
    #[error("An encoded pane screen's JSON could not be read: {0}")]
    EncodingJson(#[from] serde_json::Error),
    #[error("A pane screen scroll asked for an impossible number of lines.")]
    ScrollCount,
    #[error("A pane screen scroll ended early.")]
    ScrollTruncated,
    #[error("A pane screen scroll came without the screen lines it moves.")]
    ScrollPayload,
    #[error("A pane screen scroll named rows the screen does not have.")]
    ScrollBounds,
    #[error("A pane screen update does not match the screen it belongs to.")]
    SurfaceBaseline,
    #[error("A pane screen update is larger than the size limits this app accepts.")]
    DeltaGridLimit,
    #[error("A pane screen update carried cells where its metadata belongs.")]
    DeltaMetadataCells,
    #[error("A pane screen update named cells outside the screen, out of order, or too many.")]
    DeltaSpan,
    #[error("A pane popup update does not match the popup it belongs to.")]
    DeltaPopup,
}

impl Error {
    /// I/O failures retain their original retry category; malformed data does not retry.
    pub fn kind(&self) -> io::ErrorKind {
        match self {
            Self::Io(error) => error.kind(),
            _ => io::ErrorKind::InvalidData,
        }
    }
}
