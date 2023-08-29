use clafrica_lib::text_buffer::{Cursor, Node};
use enigo::{Enigo, Key, KeyboardControllable};
use rdev::{self, Event, EventType, Key as E_Key};

pub struct Processor {
    keyboard: Enigo,
    cursor: Cursor,
}

impl Processor {
    pub fn new(map: Node, buffer_size: usize) -> Self {
        let cursor = Cursor::new(map, buffer_size);

        Self {
            keyboard: Enigo::new(),
            cursor,
        }
    }

    pub fn process(&mut self, event: Event) -> (bool, bool) {
        let character = event.name.and_then(|s| s.chars().next());
        let is_valid = character
            .map(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
            .unwrap_or_default();
        let (mut changed, mut committed) = (false, false);

        match event.event_type {
            EventType::KeyPress(E_Key::Backspace) => {
                if let Some(out) = self.cursor.undo() {
                    self.pause();
                    self.keyboard.key_up(Key::Backspace);

                    let i = out.chars().count();
                    (1..i).for_each(|_| self.keyboard.key_click(Key::Backspace));

                    // Clear the remaining code
                    while let (None, 1.., ..) = self.cursor.state() {
                        self.cursor.undo();
                    }

                    if let (Some(_in), ..) = self.cursor.state() {
                        self.keyboard.key_sequence(&_in);
                    }

                    self.resume();
                    committed = true;
                }

                changed = true;
            }
            EventType::KeyPress(
                E_Key::Unknown(_) | E_Key::ShiftLeft | E_Key::ShiftRight | E_Key::CapsLock,
            ) => {
                // println!("[ignore] {:?}", event.event_type)
            }
            EventType::ButtonPress(_) | EventType::KeyPress(_) if !is_valid => {
                self.cursor.clear();
                changed = true;
            }
            EventType::KeyPress(_) => {
                let character = character.unwrap();

                if let Some(_in) = self.cursor.hit(character) {
                    self.pause();

                    let mut prev_cursor = self.cursor.clone();
                    prev_cursor.undo();
                    self.keyboard.key_click(Key::Backspace);

                    // Remove the remaining code
                    while let (None, 1.., ..) = prev_cursor.state() {
                        prev_cursor.undo();
                        self.keyboard.key_click(Key::Backspace);
                    }

                    if let (Some(out), ..) = prev_cursor.state() {
                        (0..out.chars().count())
                            .for_each(|_| self.keyboard.key_click(Key::Backspace))
                    }

                    self.keyboard.key_sequence(&_in);
                    self.resume();
                    committed = true;
                };

                changed = true;
            }
            _ => (),
        };

        (changed, committed)
    }

    pub fn commit(&mut self, code: &str, remaining_code: &str, text: &str) {
        self.pause();
        (0..code.len() - remaining_code.len())
            .for_each(|_| self.keyboard.key_click(Key::Backspace));
        self.keyboard.key_sequence(text);
        self.resume();
        // We clear the buffer
        self.cursor.clear();
    }

    fn pause(&mut self) {
        rdev::simulate(&EventType::KeyPress(E_Key::Pause))
            .expect("We couldn't pause the listeners");
    }

    fn resume(&mut self) {
        rdev::simulate(&EventType::KeyRelease(E_Key::Pause))
            .expect("We couldn't resume the listeners");
    }

    pub fn get_input(&self) -> String {
        self.cursor
            .to_sequence()
            .into_iter()
            .filter(|c| *c != '\0')
            .collect::<String>()
    }
}
