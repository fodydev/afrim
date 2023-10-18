mod convert;
pub mod frontend;

pub use clafrica_config::Config;
use clafrica_preprocessor::{utils, Command, Preprocessor};
use clafrica_translator::Translator;
use enigo::{Enigo, KeyboardControllable};
use frontend::Frontend;
use rdev::{self, EventType, Key as E_Key};
use std::{error, sync::mpsc, thread};

/// Start the clafrica.
pub fn run(config: Config, mut frontend: impl Frontend) -> Result<(), Box<dyn error::Error>> {
    let map = utils::build_map(
        config
            .extract_data()
            .iter()
            .map(|(key, value)| vec![key.as_str(), value.as_str()])
            .collect(),
    );
    let (buffer_size, auto_commit, page_size) = config
        .core
        .as_ref()
        .map(|core| {
            (
                core.buffer_size.unwrap_or(32),
                core.auto_commit.unwrap_or(false),
                core.page_size.unwrap_or(10),
            )
        })
        .unwrap_or((32, false, 10));
    let mut keyboard = Enigo::new();
    let mut preprocessor = Preprocessor::new(map, buffer_size);
    let translator = Translator::new(
        config.extract_translation(),
        config.extract_translators()?,
        auto_commit,
    );
    let mut is_special_pressed = false;

    frontend.set_page_size(page_size);
    frontend.update_screen(rdev::display_size().unwrap());

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut idle = false;
        let mut pause_counter = 0;

        rdev::listen(move |event| {
            idle = match event.event_type {
                EventType::KeyPress(E_Key::Pause) => true,
                EventType::KeyRelease(E_Key::Pause) => false,
                EventType::KeyPress(E_Key::ControlLeft) => idle,
                EventType::KeyRelease(E_Key::ControlLeft) => {
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
            EventType::KeyPress(E_Key::ControlLeft) => {
                is_special_pressed = true;
            }
            EventType::KeyRelease(E_Key::ControlLeft) => {
                is_special_pressed = false;
            }
            EventType::KeyRelease(E_Key::ShiftRight) if is_special_pressed => {
                frontend.next_predicate()
            }
            EventType::KeyRelease(E_Key::ShiftLeft) if is_special_pressed => {
                frontend.previous_predicate()
            }
            EventType::KeyRelease(E_Key::ControlRight) if is_special_pressed => {
                rdev::simulate(&EventType::KeyRelease(E_Key::ControlLeft))
                    .expect("We couldn't cancel the special function key");
                is_special_pressed = false;

                if let Some((_code, _remaining_code, text)) = frontend.get_selected_predicate() {
                    preprocessor.commit(text);
                    frontend.clear_predicates();
                }
            }
            _ if is_special_pressed => (),
            _ => {
                let (changed, _committed) = preprocessor.process(convert::from_event(event));

                if changed {
                    let input = preprocessor.get_input();

                    frontend.clear_predicates();

                    translator.translate(&input).iter().for_each(
                        |(code, remaining_code, texts, translated)| {
                            texts.iter().for_each(|text| {
                                if auto_commit && *translated {
                                    preprocessor.commit(text);
                                } else if !text.is_empty() {
                                    frontend.add_predicate(code, remaining_code, text);
                                }
                            });
                        },
                    );

                    frontend.set_input(&input);
                    frontend.display();
                }
            }
        }

        // Process preprocessor instructions
        while let Some(command) = preprocessor.pop_stack() {
            match command {
                Command::CommitText(text) => {
                    keyboard.key_sequence(&text);
                }
                Command::KeyPress(key) => {
                    keyboard.key_down(convert::to_key(key));
                }
                Command::KeyRelease(key) => {
                    keyboard.key_up(convert::to_key(key));
                }
                Command::KeyClick(key) => {
                    keyboard.key_click(convert::to_key(key));
                }
                Command::Pause => {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                }
                Command::Resume => {
                    rdev::simulate(&EventType::KeyRelease(E_Key::Pause)).unwrap();
                }
            };
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{frontend::Console, run, Config};
    use rdev::{self, Button, EventType::*, Key::*};
    use rstk::{self, TkPackLayout};
    use std::{thread, time::Duration};

    macro_rules! input {
        ( $( $key:expr )*, $delay:expr ) => (
            $(
                thread::sleep($delay);
                rdev::simulate(&KeyPress($key)).unwrap();
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
            run(test_config, Console::default()).unwrap();
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
        rstk::tell_wish(
            r#"
            chan configure stdout -encoding utf-8;
            wm protocol . WM_DELETE_WINDOW {destroy .};
        "#,
        );
        thread::sleep(Duration::from_secs(1));
        input_field
    }

    #[test]
    fn test_simple() {
        let typing_speed_ms = Duration::from_millis(500);

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
        input!(ControlLeft ControlLeft, typing_speed_ms);
        input!(KeyA KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑ"));
        input!(Escape, typing_speed_ms);

        // We verify the auto capitalization works as expected
        input!(CapsLock KeyA CapsLock KeyF, typing_speed_ms);
        input!(CapsLock KeyA CapsLock KeyF KeyF, typing_speed_ms);
        input!(KeyA KeyF KeyF, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑ"));
        input!(Escape, typing_speed_ms);

        // We verify that the translation work as expected
        input!(KeyH KeyE KeyL KeyL KeyO, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhi"));
        input!(Escape KeyH KeyI, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihello"));
        input!(Escape, typing_speed_ms);

        // We verify that the predicate selection work as expected
        input!(KeyH KeyE, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(ShiftLeft, typing_speed_ms);
        input!(ShiftRight, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();

        input!(KeyL KeyL, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(ControlRight, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohi"));
        input!(Escape, typing_speed_ms);

        // We verify that we don't have a conflict
        // between the translator and the processor
        input!(KeyV KeyU KeyU KeyE, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohivʉe"));
    }
}
