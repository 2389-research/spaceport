//! A client's copy of the transcript, kept in step with the change stream.

use crate::{Change, Cursor, Error, Result, Touched, Transcript};

/// The transcript as of one position in the program's change stream.
///
/// The position and the transcript only change together, so a position is
/// held only while the transcript reflects exactly the changes up to it.
#[derive(Debug, Default)]
pub struct Replica {
    /// The last change applied, or `None` when only a snapshot can bring
    /// the transcript back in step.
    cursor: Option<Cursor>,
    transcript: Transcript,
}

impl Replica {
    /// Starts over from a snapshot taken at `cursor`.
    pub fn reset(&mut self, cursor: Cursor, transcript: Transcript) {
        self.cursor = Some(cursor);
        self.transcript = transcript;
    }

    /// Applies the change at `cursor` and reports what it touched. It must be
    /// the one right after the position held: the same epoch and the next
    /// sequence number.
    ///
    /// A change that does not follow, or does not fit the transcript, leaves
    /// the transcript as it was and forgets the position: the transcript can
    /// stay on screen, and the next connection asks for a snapshot.
    pub fn follow(&mut self, change: &Change, cursor: Cursor) -> Result<Touched> {
        let after = self.cursor.take();
        let follows = after.is_some_and(|after| {
            after.epoch == cursor.epoch && after.seq.checked_add(1) == Some(cursor.seq)
        });
        if !follows {
            return Err(Error::StreamGap { after, got: cursor });
        }
        self.transcript.apply(change)?;
        self.cursor = Some(cursor);
        Ok(change.touched())
    }

    /// Forgets the position and keeps the transcript, so the next connection
    /// asks for a snapshot.
    pub fn lose(&mut self) {
        self.cursor = None;
    }

    pub fn transcript(&self) -> &Transcript {
        &self.transcript
    }

    /// The transcript, for what a client knows from outside the stream, such
    /// as the status a daemon reports for an observed agent. The position
    /// stays, so the next snapshot or change may overwrite the edit.
    pub fn transcript_mut(&mut self) -> &mut Transcript {
        &mut self.transcript
    }

    /// The position to resume from when reconnecting, or `None` to ask for a
    /// snapshot.
    pub fn since(&self) -> Option<Cursor> {
        self.cursor
    }
}
