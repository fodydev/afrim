#![deny(missing_docs)]

use keyboard_types::Key;

/// Possible commands that can be generated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    /// Request to commit a text.
    CommitText(String),
    /// Request to pause the listener.
    Pause,
    /// Request to resume the listener.
    Resume,
    /// Request to press a key.
    KeyPress(Key),
    /// Request to release a key.
    KeyRelease(Key),
    /// Request to toggle a key.
    KeyClick(Key),
}
