mod message;

pub use crate::message::Command;
pub use clafrica_memory::utils;
use clafrica_memory::{Cursor, Node};
pub use keyboard_types::{Key, KeyState, KeyboardEvent};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Preprocessor {
    cursor: Cursor,
    stack: VecDeque<Command>,
}

impl Preprocessor {
    pub fn new(map: Node, buffer_size: usize) -> Self {
        let cursor = Cursor::new(map, buffer_size);
        let stack = VecDeque::with_capacity(15);

        Self { cursor, stack }
    }

    fn rollback(&mut self) -> bool {
        self.stack.push_back(Command::KeyRelease(Key::Backspace));

        if let Some(out) = self.cursor.undo() {
            (1..out.chars().count())
                .for_each(|_| self.stack.push_back(Command::KeyClick(Key::Backspace)));

            // Clear the remaining code
            while let (None, 1.., ..) = self.cursor.state() {
                self.cursor.undo();
            }

            if let (Some(_in), ..) = self.cursor.state() {
                self.stack.push_back(Command::CommitText(_in));
            }

            true
        } else {
            false
        }
    }

    pub fn process(&mut self, event: KeyboardEvent) -> (bool, bool) {
        let (mut changed, mut committed) = (false, false);

        match (event.state, event.key) {
            (KeyState::Down, Key::Backspace) => {
                self.pause();
                committed = self.rollback();
                self.resume();
                changed = true;
            }
            (KeyState::Down, Key::Character(character))
                if character
                    .chars()
                    .next()
                    .map(|e| e.is_alphanumeric() || e.is_ascii_punctuation())
                    .unwrap_or(false) =>
            {
                let character = character.chars().next().unwrap();

                if let Some(_in) = self.cursor.hit(character) {
                    self.pause();

                    let mut prev_cursor = self.cursor.clone();
                    prev_cursor.undo();
                    self.stack.push_back(Command::KeyClick(Key::Backspace));

                    // Remove the remaining code
                    while let (None, 1.., ..) = prev_cursor.state() {
                        prev_cursor.undo();
                        self.stack.push_back(Command::KeyClick(Key::Backspace));
                    }

                    if let (Some(out), ..) = prev_cursor.state() {
                        (0..out.chars().count())
                            .for_each(|_| self.stack.push_back(Command::KeyClick(Key::Backspace)))
                    }

                    self.stack.push_back(Command::CommitText(_in));
                    self.resume();
                    committed = true;
                };

                changed = true;
            }
            (KeyState::Down, Key::Shift | Key::CapsLock) => (),
            (KeyState::Down, _) => {
                self.cursor.clear();
                changed = true;
            }
            _ => (),
        };

        (changed, committed)
    }

    pub fn commit(&mut self, text: &str) {
        self.pause();

        while !self.cursor.is_empty() {
            self.stack.push_back(Command::KeyPress(Key::Backspace));
            self.rollback();
        }
        self.stack.push_back(Command::CommitText(text.to_owned()));
        self.resume();
        // We clear the buffer
        self.cursor.clear();
    }

    fn pause(&mut self) {
        self.stack.push_back(Command::Pause);
    }

    fn resume(&mut self) {
        self.stack.push_back(Command::Resume);
    }

    pub fn get_input(&self) -> String {
        self.cursor
            .to_sequence()
            .into_iter()
            .filter(|c| *c != '\0')
            .collect::<String>()
    }

    pub fn pop_stack(&mut self) -> Option<Command> {
        self.stack.pop_front()
    }

    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::message::Command;
    use crate::utils;
    use crate::Preprocessor;
    use keyboard_types::{
        webdriver::{self, Event},
        Key::*,
    };
    use std::collections::VecDeque;
    use std::fs;

    #[test]
    fn test_process() {
        let data = utils::load_data("ccced ç\ncc ç");
        let map = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(map, 8);
        webdriver::send_keys("\u{E00C}ccced")
            .into_iter()
            .for_each(|e| {
                match e {
                    Event::Keyboard(e) => preprocessor.process(e),
                    _ => unimplemented!(),
                };
            });
        let mut expecteds = VecDeque::from(vec![
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_stack() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
    }

    #[test]
    fn test_commit() {
        use clafrica_memory::Node;
        use keyboard_types::KeyboardEvent;

        let mut preprocessor = Preprocessor::new(Node::default(), 8);
        preprocessor.process(KeyboardEvent {
            key: Character("a".to_owned()),
            ..Default::default()
        });
        preprocessor.commit("word");

        let mut expecteds = VecDeque::from(vec![
            Command::Pause,
            Command::KeyPress(Backspace),
            Command::KeyRelease(Backspace),
            Command::CommitText("word".to_owned()),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_stack() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
    }

    #[test]
    fn test_rollback() {
        use keyboard_types::KeyboardEvent;

        let data = utils::load_data("ccced ç\ncc ç");
        let map = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(map, 8);
        let backspace_event = KeyboardEvent {
            key: Backspace,
            ..Default::default()
        };

        webdriver::send_keys("ccced").into_iter().for_each(|e| {
            match e {
                Event::Keyboard(e) => preprocessor.process(e),
                _ => unimplemented!(),
            };
        });

        preprocessor.clear_stack();
        assert_eq!(preprocessor.get_input(), "ccced".to_owned());
        preprocessor.process(backspace_event.clone());
        assert_eq!(preprocessor.get_input(), "cc".to_owned());
        preprocessor.process(backspace_event);
        assert_eq!(preprocessor.get_input(), "".to_owned());

        let mut expecteds = VecDeque::from(vec![
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_stack() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
    }

    #[test]
    fn test_advanced() {
        let data = fs::read_to_string("./data/sample.txt").unwrap();
        let data = utils::load_data(&data);
        let map = utils::build_map(data);
        let mut preprocessor = Preprocessor::new(map, 64);

        webdriver::send_keys(
            "u\u{E003}uu\u{E003}uc_ceduuaf3afafaff3uu3\
            \u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}\u{E003}"
        ).into_iter().for_each(|e| {
            match e {
                Event::Keyboard(e) => preprocessor.process(e),
                _ => unimplemented!(),
            };
        });

        let mut expecteds = VecDeque::from(vec![
            // Process
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑ\u{304}ɑ\u{304}".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ\u{304}".to_owned()),
            Command::Resume,
            // Rollback
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ɑ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::KeyClick(Backspace),
            Command::CommitText("ʉ".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::CommitText("ç".to_owned()),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
            Command::Pause,
            Command::KeyRelease(Backspace),
            Command::Resume,
        ]);

        while let Some(command) = preprocessor.pop_stack() {
            assert_eq!(command, expecteds.pop_front().unwrap());
        }
    }
}
