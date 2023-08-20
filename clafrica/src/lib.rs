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
            .iter()
            .map(|(k, v)| [k.as_str(), v.as_str()])
            .collect(),
    );
    let mut cursor = text_buffer::Cursor::new(map, config.core.map(|e| e.buffer_size).unwrap_or(8));

    let mut keyboard = Enigo::new();

    frontend.update_screen(rdev::display_size().unwrap());

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut idle = false;
        let mut pause_counter = 0;

        rdev::listen(move |event| {
            idle = match event.event_type {
                EventType::KeyPress(E_Key::Pause) => true,
                EventType::KeyRelease(E_Key::Pause) => false,
                EventType::KeyPress(E_Key::ControlLeft | E_Key::ControlRight) => idle,
                EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) => {
                    pause_counter += 1;

                    if pause_counter != 0 && pause_counter % 2 == 0 {
                        pause_counter = 0;
                        !idle
                    } else {
                        idle
                    }
                }
                _ => {
                    pause_counter = 0;
                    idle
                }
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
                    while let (None, 1.., ..) = cursor.state() {
                        cursor.undo();
                    }

                    if let (Some(_in), ..) = cursor.state() {
                        keyboard.key_sequence(&_in);
                    }
                }

                frontend.update_text(cursor.to_sequence());
            }
            EventType::KeyPress(
                E_Key::Unknown(_) | E_Key::ShiftLeft | E_Key::ShiftRight | E_Key::CapsLock,
            ) => {
                // println!("[ignore] {:?}", event.event_type)
            }
            EventType::ButtonPress(_) | EventType::KeyPress(_) if !is_valid => {
                cursor.clear();
                frontend.update_text(cursor.to_sequence());
            }
            EventType::KeyPress(_) => {
                let character = character.unwrap();

                let mut prev_cursor = cursor.clone();

                if let Some(_in) = cursor.hit(character) {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause))
                        .expect("We could pause the listeners");

                    keyboard.key_click(Key::Backspace);

                    // Remove the remaining code
                    while let (None, 1.., ..) = prev_cursor.state() {
                        prev_cursor.undo();
                        keyboard.key_click(Key::Backspace);
                    }

                    if let (Some(out), ..) = prev_cursor.state() {
                        (0..out.chars().count()).for_each(|_| keyboard.key_click(Key::Backspace))
                    }

                    keyboard.key_sequence(&_in);

                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause))
                        .expect("We could resume the listeners");
                };

                frontend.update_text(cursor.to_sequence());
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
mod tests {
    use crate::{api, config::Config, run};
    use rdev::{self, Button, EventType::*, Key::*};
    use rstk::{self, TkPackLayout};
    use std::{thread, time::Duration};

    macro_rules! input {
        ( $( $key:expr )*, $delay:expr ) => (
            $(
                thread::sleep($delay);
                rdev::simulate(&KeyPress($key)).unwrap();
                thread::sleep($delay);
                rdev::simulate(&KeyRelease($key)).unwrap();
            )*
        );
    }

    macro_rules! output {
        ( $textfield: expr, $expected: expr ) => {
            // A loop to be sure to got something stable
            loop {
                let a = $textfield.get_to_end((1, 0));
                let b = $textfield.get_to_end((1, 0));

                if (a == b) {
                    let content = a.chars().filter(|c| *c != '\0').collect::<String>();
                    let content = content.trim();

                    assert_eq!(content, $expected);
                    break;
                }
            }
        };
    }

    fn start_clafrica() {
        use std::path::Path;

        let test_config = Config::from_file(Path::new("./data/test.toml")).unwrap();

        thread::spawn(move || {
            run(test_config, api::Console).unwrap();
        });
    }

    fn start_sandbox() -> rstk::TkText {
        let root = rstk::trace_with("wish").unwrap();
        root.title("Clafrica Test Environment");
        let input_field = rstk::make_text(&root);
        input_field.width(50);
        input_field.height(12);
        input_field.pack().layout();
        root.geometry(200, 200, 0, 0);
        rstk::tell_wish("chan configure stdout -encoding utf-8;");
        thread::sleep(Duration::from_secs(1));
        input_field
    }

    #[test]
    fn test_simple() {
        let typing_speed_ms = Duration::from_millis(300);

        // To detect excessive backspace
        const LIMIT: &str = "bbb";

        // Start the clafrica
        start_clafrica();

        // Start the sandbox
        let textfield = start_sandbox();

        rdev::simulate(&MouseMove { x: 100.0, y: 100.0 }).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonPress(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonRelease(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);

        input!(KeyB KeyB KeyB Escape, typing_speed_ms);
        input!(KeyU Backspace KeyU KeyU Backspace KeyU, typing_speed_ms);
        input!(
            KeyC Num8 KeyC KeyE KeyD
            KeyU KeyU
            KeyA KeyF Num3, typing_speed_ms);
        input!(
            KeyA KeyF KeyA KeyF
            KeyA KeyF KeyF Num3, typing_speed_ms);
        input!(KeyU KeyU Num3, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uçʉ̄ɑ̄ɑɑɑ̄ɑ̄ʉ̄"));

        // We verify that the undo (backspace) works as expected
        (0..12).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });
        output!(textfield, LIMIT);

        // We verify that the pause/resume works as expected
        input!(ControlLeft ControlLeft, typing_speed_ms);
        input!(KeyU KeyU, typing_speed_ms);
        input!(ControlLeft ControlRight, typing_speed_ms);
        input!(KeyA KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑ"));

        (0..3).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });

        // We verify the auto capitalization works as expected
        input!(CapsLock KeyA CapsLock KeyF, typing_speed_ms);
        input!(CapsLock KeyA CapsLock KeyF KeyF, typing_speed_ms);
        input!(KeyA KeyF KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}αⱭⱭɑɑ"));

        rstk::end_wish();
    }
}
