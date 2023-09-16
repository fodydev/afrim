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

    fn rollback(&mut self) -> bool {
        self.keyboard.key_up(Key::Backspace);

        if let Some(out) = self.cursor.undo() {
            (1..out.chars().count()).for_each(|_| self.keyboard.key_click(Key::Backspace));

            // Clear the remaining code
            while let (None, 1.., ..) = self.cursor.state() {
                self.cursor.undo();
            }

            if let (Some(_in), ..) = self.cursor.state() {
                self.keyboard.key_sequence(&_in);
            }

            true
        } else {
            false
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
                self.pause();
                committed = self.rollback();
                self.resume();
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

    pub fn commit(&mut self, text: &str) {
        self.pause();
        while !self.cursor.is_empty() {
            self.keyboard.key_down(Key::Backspace);
            self.rollback();
        }
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
