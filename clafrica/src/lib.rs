pub mod api;
pub mod config;

use crate::api::Frontend;
use clafrica_lib::{text_buffer, utils};
use enigo::{Enigo, Key, KeyboardControllable};
use rdev::{self, EventType, Key as E_Key};
use std::{io, sync::mpsc, thread};

pub mod prelude {
    pub use crate::config::Config;
}

pub fn run(config: config::Config, mut frontend: impl Frontend) -> Result<(), io::Error> {
    let map = utils::build_map(
        config
            .extract_data()
            .into_iter()
            .map(|(k, v)| [k.as_str(), v.as_str()])
            .collect(),
    );
    let mut cursor = text_buffer::Cursor::new(map, config.core.map(|e| e.buffer_size).unwrap_or(8));

    let mut keyboard = Enigo::new();

    frontend.update_screen(rdev::display_size().unwrap());

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut idle = false;

        rdev::listen(move |event| {
            idle = match event.event_type {
                EventType::KeyPress(E_Key::Pause) => true,
                EventType::KeyRelease(E_Key::Pause) => false,
                _ => idle,
            };
            if !idle {
                tx.send(event)
                    .unwrap_or_else(|e| eprintln!("Could not send event {:?}", e));
            }
        })
        .expect("Could not listen");
    });

    for event in rx.iter() {
        let character = event.name.and_then(|s| s.chars().next());
        let is_valid = character
            .map(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
            .unwrap_or_default();

        match event.event_type {
            EventType::KeyPress(E_Key::Backspace) => {
                if let Some(out) = cursor.undo() {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause))
                        .expect("We could pause the listeners");
                    keyboard.key_up(Key::Backspace);

                    let i = out.chars().count();
                    (1..i).for_each(|_| keyboard.key_click(Key::Backspace));

                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause))
                        .expect("We could resume the listeners");

                    // Clear the remaining code
                    while let (None, _i @ 1.., ..) = cursor.state() {
                        cursor.undo();
                    }

                    if let (Some(prev_out), ..) = cursor.state() {
                        keyboard.key_sequence(&prev_out);
                    }
                }

                frontend.update_text(cursor.to_path());
            }
            EventType::KeyPress(E_Key::Unknown(_) | E_Key::ShiftLeft | E_Key::ShiftRight) => {
                // println!("[ignore] {:?}", event.event_type)
            }
            EventType::ButtonPress(_) | EventType::KeyPress(_) if !is_valid => {
                cursor.clear();
                frontend.update_text(cursor.to_path());
            }
            EventType::KeyPress(_) => {
                let character = character.unwrap();

                let (prev_out, prev_code_len, ..) = cursor.state();
                let out = cursor.hit(character);

                if let Some(out) = out {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause))
                        .expect("We could pause the listenerss");

                    let i = prev_out.map(|s| s.chars().count()).unwrap_or(prev_code_len) + 1;
                    (0..i).for_each(|_| keyboard.key_click(Key::Backspace));
                    keyboard.key_sequence(&out);

                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause))
                        .expect("We could resume the listeners");
                };

                frontend.update_text(cursor.to_path());
            }
            EventType::MouseMove { x, y } => {
                frontend.update_position((x, y));
            }
            _ => (),
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
