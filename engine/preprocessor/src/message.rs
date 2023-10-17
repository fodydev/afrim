use keyboard_types::Key;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    CommitText(String),
    Pause,
    Resume,
    KeyPress(Key),
    KeyRelease(Key),
    KeyClick(Key),
}
