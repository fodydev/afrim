use clafrica_lib::{text_buffer, utils};
use enigo::{Enigo, Key, KeyboardControllable};
use rdev::{self, EventType, Key as E_Key};
use std::sync::mpsc;
use std::thread;
use std::{env, io};

pub struct Config {
    data_path: String,
    buffer_size: usize,
}

impl Config {
    pub fn build(mut args: env::Args) -> Result<Self, &'static str> {
        args.next();
        Ok(Self {
            data_path: args.next().ok_or("filepath required")?,
            buffer_size: 10,
        })
    }
}

pub fn run(config: Config) -> Result<(), io::Error> {
    let data = utils::load_data(&config.data_path)?;
    let map = utils::build_map(
        data.iter()
            .map(|e| [e[0].as_ref(), e[1].as_ref()])
            .collect(),
    );
    let mut cursor = text_buffer::Cursor::new(map, config.buffer_size);

    let mut keyboard = Enigo::new();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut idle = false;

        rdev::listen(move |event| {
            idle = match event.event_type {
                EventType::KeyPress(E_Key::Escape) => true,
                EventType::KeyRelease(E_Key::Escape) => false,
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
            .map(|c| c.is_ascii() && !c.is_whitespace())
            .unwrap_or_default();

        match event.event_type {
            EventType::KeyPress(E_Key::Backspace) => {
                if let Some(out) = cursor.undo() {
                    keyboard.key_down(Key::Escape);

                    let i = out.chars().count();
                    (0..i).for_each(|_| keyboard.key_click(Key::Backspace));

                    if let (Some(prev_out), ..) = cursor.state() {
                        keyboard.key_sequence(&prev_out);
                    }

                    keyboard.key_up(Key::Escape);

                    // Clear the remaining code
                    while let (None, _i @ 1.., _c) = cursor.state() {
                        cursor.undo();
                    }
                }
            }
            EventType::KeyPress(E_Key::Unknown(_) | E_Key::ShiftLeft | E_Key::ShiftRight) => {
                println!("[ignore] {:?}", event.event_type)
            }
            EventType::ButtonPress(_) | EventType::KeyPress(_) if !is_valid => {
                cursor.clear();
                println!("Buffer cleared");
            }
            EventType::KeyPress(_) => {
                let character = character.unwrap();
                println!("Received: {:?}", character);

                let (prev_out, prev_code_len, ..) = cursor.state();
                let out = cursor.hit(character);

                if let Some(out) = out {
                    keyboard.key_down(Key::Escape);

                    let i = prev_out.map(|s| s.chars().count()).unwrap_or(prev_code_len) + 1;
                    (0..i).for_each(|_| keyboard.key_click(Key::Backspace));
                    keyboard.key_sequence(&out);

                    keyboard.key_up(Key::Escape);
                };

                println!("{:?}", cursor.to_path());
            }
            _ => (),
        };
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
