mod convert;
pub mod frontend;

pub use afrim_config::Config;
use afrim_preprocessor::{utils, Command as EventCmd, Preprocessor};
use afrim_translator::Translator;
use anyhow::{Context, Result};
use enigo::{Enigo, Key, KeyboardControllable};
use frontend::{Command as GUICmd, Frontend};
use rdev::{self, EventType, Key as E_Key};
use std::{rc::Rc, sync::mpsc, thread};

/// Starts the afrim.
pub fn run(
    config: Config,
    mut frontend: impl Frontend + std::marker::Send + 'static,
) -> Result<()> {
    // State.
    let mut is_ctrl_released = true;
    let mut idle = false;

    // Configuration of the afrim.
    let memory = utils::build_map(
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
    let mut preprocessor = Preprocessor::new(Rc::new(memory), buffer_size);
    #[cfg(not(feature = "rhai"))]
    let translator = Translator::new(config.extract_translation(), auto_commit);
    #[cfg(feature = "rhai")]
    let mut translator = Translator::new(config.extract_translation(), auto_commit);
    #[cfg(feature = "rhai")]
    config
        .extract_translators()
        .context("Failed to load translators.")?
        .into_iter()
        .for_each(|(name, ast)| translator.register(name, ast));

    // Configuration of the frontend.
    let (frontend_tx1, frontend_rx1) = mpsc::channel();
    let (frontend_tx2, frontend_rx2) = mpsc::channel();

    frontend_tx1.send(GUICmd::PageSize(page_size))?;
    let screen_size = rdev::display_size().unwrap();
    frontend_tx1.send(GUICmd::ScreenSize(screen_size))?;

    thread::spawn(move || {
        frontend.set_channel(frontend_tx2, frontend_rx1);
        frontend
            .listen()
            .context("Problem of handling GUI commands.")
            .unwrap_or_else(|e| eprintln!("{:?}", e));
    });

    // Configuration of the event listener.
    let (event_tx, event_rx) = mpsc::channel();
    thread::spawn(move || {
        rdev::listen(move |event| {
            event_tx
                .send(event)
                .unwrap_or_else(|e| eprintln!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });

    // We process event.
    for event in event_rx.iter() {
        match event.event_type {
            // Handling of idle state.
            EventType::KeyPress(E_Key::Pause) => {
                idle = true;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyRelease(E_Key::Pause) => {
                idle = false;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyPress(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_ctrl_released = false;
            }
            EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) if is_ctrl_released => {
                idle = !idle;
                frontend_tx1.send(GUICmd::State(idle))?;
            }
            EventType::KeyRelease(E_Key::ControlLeft | E_Key::ControlRight) => {
                is_ctrl_released = true;
            }
            _ if idle => (),
            // Handling of special functions.
            EventType::KeyRelease(E_Key::ShiftRight) if !is_ctrl_released => {
                frontend_tx1.send(GUICmd::SelectNextPredicate)?;
            }
            EventType::KeyRelease(E_Key::ShiftLeft) if !is_ctrl_released => {
                frontend_tx1.send(GUICmd::SelectPreviousPredicate)?;
            }
            EventType::KeyRelease(E_Key::Space) if !is_ctrl_released => {
                rdev::simulate(&EventType::KeyRelease(E_Key::ControlLeft))
                    .expect("We couldn't cancel the special function key");

                frontend_tx1.send(GUICmd::SelectedPredicate)?;
                if let GUICmd::Predicate(predicate) = frontend_rx2.recv()? {
                    preprocessor.commit(
                        predicate
                            .texts
                            .first()
                            .unwrap_or(&String::default())
                            .to_owned(),
                    );
                    frontend_tx1.send(GUICmd::Clear)?;
                }
            }
            _ if !is_ctrl_released => (),
            // GUI events.
            EventType::MouseMove { x, y } => {
                frontend_tx1.send(GUICmd::Position((x, y)))?;
            }
            // Process events.
            _ => {
                let (changed, _committed) = preprocessor.process(convert::from_event(event));

                if changed {
                    let input = preprocessor.get_input();

                    frontend_tx1.send(GUICmd::Clear)?;

                    translator
                        .translate(&input)
                        .into_iter()
                        .take(page_size * 2)
                        .try_for_each(|predicate| -> Result<()> {
                            if predicate.texts.is_empty() {
                            } else if auto_commit && predicate.can_commit {
                                preprocessor.commit(predicate.texts[0].to_owned());
                            } else {
                                frontend_tx1.send(GUICmd::Predicate(predicate))?;
                            }

                            Ok(())
                        })?;

                    frontend_tx1.send(GUICmd::InputText(input))?;
                    frontend_tx1.send(GUICmd::Update)?;
                }
            }
        }

        // Process preprocessor instructions
        while let Some(command) = preprocessor.pop_queue() {
            match command {
                EventCmd::CommitText(text) => {
                    keyboard.key_sequence(&text);
                }
                EventCmd::CleanDelete => {
                    keyboard.key_up(Key::Backspace);
                }
                EventCmd::Delete => {
                    keyboard.key_click(Key::Backspace);
                }
                EventCmd::Pause => {
                    rdev::simulate(&EventType::KeyPress(E_Key::Pause)).unwrap();
                }
                EventCmd::Resume => {
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

    fn start_afrim() {
        use std::path::Path;

        let test_config = Config::from_file(Path::new("./data/test.toml")).unwrap();

        thread::spawn(move || {
            run(test_config, Console::default()).unwrap();
        });
    }

    fn start_sandbox(start_point: &str) -> rstk::TkText {
        let root = rstk::trace_with("wish").unwrap();
        root.title("Afrim Test Environment");

        let input_field = rstk::make_text(&root);
        input_field.width(50);
        input_field.height(12);
        input_field.pack().layout();
        root.geometry(200, 200, 0, 0);
        input_field.insert((1, 1), start_point);
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

        // Start the afrim
        start_afrim();

        // Start the sandbox
        let textfield = start_sandbox(LIMIT);

        rdev::simulate(&MouseMove { x: 100.0, y: 100.0 }).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonPress(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);
        rdev::simulate(&ButtonRelease(Button::Left)).unwrap();
        thread::sleep(typing_speed_ms);

        input!(KeyU, typing_speed_ms);
        #[cfg(not(feature = "inhibit"))]
        input!(Backspace, typing_speed_ms);
        input!(KeyU KeyU Backspace KeyU, typing_speed_ms);
        input!(
            KeyC Num8 KeyC KeyE KeyD
            KeyU KeyU
            KeyA KeyF Num3, typing_speed_ms);
        input!(
            KeyA KeyF KeyA KeyF
            KeyA KeyF KeyF Num3, typing_speed_ms);
        input!(KeyU KeyU Num3, typing_speed_ms);
        #[cfg(feature = "inhibit")]
        output!(textfield, format!("{LIMIT}çʉ̄ɑ̄ɑɑɑ̄ɑ̄ʉ̄"));
        #[cfg(not(feature = "inhibit"))]
        output!(textfield, format!("{LIMIT}uçʉ̄ɑ̄ɑɑɑ̄ɑ̄ʉ̄"));

        // We verify that the undo (backspace) works as expected
        #[cfg(not(feature = "inhibit"))]
        (0..12).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });
        #[cfg(feature = "inhibit")]
        (0..13).for_each(|_| {
            input!(Backspace, typing_speed_ms);
        });
        output!(textfield, LIMIT);

        // We verify that the pause/resume works as expected
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        rdev::simulate(&KeyPress(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        input!(KeyU KeyU, typing_speed_ms);

        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        rdev::simulate(&KeyPress(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlRight)).unwrap();
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
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
        #[cfg(not(feature = "rhai"))]
        input!(Escape KeyH Escape KeyE KeyL KeyL KeyO, typing_speed_ms);
        #[cfg(feature = "rhai")]
        input!(Escape KeyH KeyI, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihello"));
        input!(Escape, typing_speed_ms);

        // We verify that the predicate selection work as expected
        input!(KeyH KeyE, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(ShiftLeft, typing_speed_ms);
        input!(ShiftRight, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();

        input!(KeyA, typing_speed_ms);
        rdev::simulate(&KeyPress(ControlLeft)).unwrap();
        input!(Space, typing_speed_ms);
        rdev::simulate(&KeyRelease(ControlLeft)).unwrap();
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohealth"));
        input!(Escape, typing_speed_ms);

        // We verify that we don't have a conflict
        // between the translator and the processor
        input!(KeyV KeyU KeyU KeyE, typing_speed_ms);
        output!(textfield, format!("{LIMIT}uuɑαⱭⱭɑɑhihellohealthvʉe"));
    }
}
