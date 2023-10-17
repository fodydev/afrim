use clafrica_preprocessor::{Key, KeyState, KeyboardEvent};
use enigo::{self};
use rdev::{self};

/// Converts an rdev::Event into a KeyboardEvent.
pub fn from_event(event: rdev::Event) -> KeyboardEvent {
    let key_char = event
        .name
        .and_then(|c| c.chars().next())
        .filter(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
        .map(|c| Key::Character(c.to_string()));
    let (state, key) = match event.event_type {
        rdev::EventType::KeyPress(key) => (KeyState::Down, from_key(key)),
        rdev::EventType::KeyRelease(key) => (KeyState::Up, from_key(key)),
        _ => Default::default(),
    };

    KeyboardEvent {
        key: key_char.unwrap_or(key),
        state,
        ..Default::default()
    }
}

/// Converts an rdev::Key into a Key.
pub fn from_key(key: rdev::Key) -> Key {
    match key {
        rdev::Key::Alt => Key::Alt,
        rdev::Key::AltGr => Key::AltGraph,
        rdev::Key::Backspace => Key::Backspace,
        rdev::Key::CapsLock => Key::CapsLock,
        rdev::Key::ControlLeft => Key::Control,
        rdev::Key::ControlRight => Key::Control,
        rdev::Key::ShiftLeft => Key::Shift,
        rdev::Key::ShiftRight => Key::Shift,
        rdev::Key::ScrollLock => Key::ScrollLock,
        rdev::Key::Pause => Key::Pause,
        rdev::Key::NumLock => Key::NumLock,
        rdev::Key::Insert => Key::Insert,
        _ => Default::default(),
    }
}

/// Converts a Key into an enigo::Key.
pub fn to_key(key: Key) -> enigo::Key {
    match key {
        Key::Backspace => enigo::Key::Backspace,
        _ => unimplemented!(),
    }
}
