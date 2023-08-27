pub mod api;
mod config;
mod processor;
mod translator;

use crate::api::Frontend;
use crate::processor::Processor;
use crate::translator::Translator;
use clafrica_lib::utils;
use rdev::{self, EventType, Key as E_Key};
use std::{error, sync::mpsc, thread};

pub mod prelude {
    pub use crate::config::Config;
}

pub fn run(
    config: config::Config,
    mut frontend: impl Frontend,
) -> Result<(), Box<dyn error::Error>> {
    let map = utils::build_map(
        config
            .extract_data()
            .iter()
            .map(|(k, v)| [k.as_str(), v.as_str()])
            .collect(),
    );
    let mut processor = Processor::new(
        map,
        config.core.as_ref().map(|e| e.buffer_size).unwrap_or(8),
    );
    let translator = Translator::new(
        config.extract_translation(),
        config.core.as_ref().map(|e| e.auto_commit).unwrap_or(false),
    );
    let mut is_special_pressed = false;

    frontend.set_page_size(config.core.as_ref().map(|e| e.page_size).unwrap_or(10));
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
        match event.event_type {
            EventType::MouseMove { x, y } => {
                frontend.update_position((x, y));
            }
            EventType::KeyPress(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_special_pressed = true;
            }
            EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_special_pressed = false;
            }
            EventType::KeyRelease(E_Key::Alt) if is_special_pressed => frontend.next_predicate(),
            EventType::KeyRelease(E_Key::Unknown(151)) if is_special_pressed => {
                frontend.previous_predicate()
            }
            EventType::KeyRelease(E_Key::Space) if is_special_pressed => {
                if let Some(predicate) = frontend.get_selected_predicate() {
                    is_special_pressed = false;
                    processor.commit(&predicate.0, &predicate.1, &predicate.2);
                }
            }
            _ if is_special_pressed => (),
            _ => {
                let (changed, committed) = processor.process(event);

                if changed {
                    let input = processor.get_input();

                    frontend.clear_predicates();

                    if !committed {
                        translator.translate(&input).iter().for_each(
                            |(code, remaining_code, text, translated)| {
                                if *translated {
                                    processor.commit(code, remaining_code, text);
                                } else if !text.is_empty() {
                                    frontend.add_predicate(code, remaining_code, text);
                                }
                            },
                        );
                    };

                    frontend.set_input(&input);
                    frontend.display();
                }
            }
        }
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
            thread::sleep(Duration::from_millis(500));

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
            run(test_config, api::Console::default()).unwrap();
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

        (0..5).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });

        // We verify that the translation work as expected
        input!(KeyH KeyE KeyL KeyL KeyO, typing_speed_ms);
        output!(textfield, format!("{LIMIT}hi"));

        (0..2).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });

        // We verify that the predicate selection work as expected
        input!(KeyH KeyE, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(Unknown(151), typing_speed_ms);
        input!(Alt, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();

        input!(KeyL KeyL, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(Space, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        output!(textfield, format!("{LIMIT}hi"));

        rstk::end_wish();
    }
}
