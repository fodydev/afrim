#![deny(missing_docs)]
//! API to develop a frontend interface for the afrim.
//!

pub use afrim_translator::Predicate;

/// Trait that every afrim frontend should implement.
pub trait Frontend {
    /// Sets the screen size.
    fn set_screen_size(&mut self, _screen: (u64, u64)) {}
    /// Sets the position of the frontend.
    fn set_position(&mut self, _position: (f64, f64)) {}
    /// Sets the input text to display.
    fn set_input_text(&mut self, _text: &str) {}
    /// Sets the maximun number of predicates to display.
    fn set_max_predicates(&mut self, _size: usize) {}
    /// Adds a predicate to the list.
    fn add_predicate(&mut self, _predicate: Predicate) {}
    /// Updates the display.
    fn update_display(&mut self) {}
    /// Clears the list of predicates.
    fn clear_all_predicates(&mut self) {}
    /// Selects the previous predicate.
    fn select_previous_predicate(&mut self) {}
    /// Selects the next predicate.
    fn select_next_predicate(&mut self) {}
    /// Returns the currently selected predicate.
    fn get_selected_predicate(&self) -> Option<&Predicate> {
        Option::None
    }
    /// Sets the state of the afrim.
    fn set_state(&mut self, _state: bool) {}
}

/// This frontend do nothing.
pub struct None;

impl Frontend for None {}

/// Cli frontent interface.
#[derive(Default)]
pub struct Console {
    page_size: usize,
    predicates: Vec<Predicate>,
    current_predicate_id: usize,
    input: String,
}

impl Frontend for Console {
    fn set_max_predicates(&mut self, size: usize) {
        self.page_size = size;
        self.predicates = Vec::with_capacity(size);
    }

    fn set_input_text(&mut self, text: &str) {
        self.input = text.to_owned();
    }

    fn update_display(&mut self) {
        // Input
        println!("input: {}", self.input);

        // Predicates
        let page_size = std::cmp::min(self.page_size, self.predicates.len());
        println!(
            "Predicates: {}",
            self.predicates
                .iter()
                .enumerate()
                .chain(self.predicates.iter().enumerate())
                .skip(self.current_predicate_id)
                .take(page_size)
                .map(|(id, predicate)| {
                    format!(
                        "{}{}. {} ~{}\t ",
                        if id == self.current_predicate_id {
                            "*"
                        } else {
                            ""
                        },
                        id + 1,
                        predicate.texts[0],
                        predicate.remaining_code
                    )
                })
                .collect::<Vec<_>>()
                .join("\t")
        );
    }

    fn clear_all_predicates(&mut self) {
        self.predicates.clear();
        self.current_predicate_id = 0;
    }

    fn add_predicate(&mut self, predicate: Predicate) {
        predicate
            .texts
            .iter()
            .filter(|text| !text.is_empty())
            .for_each(|text| {
                let mut predicate = predicate.clone();
                predicate.texts = vec![text.to_owned()];

                self.predicates.push(predicate);
            });
    }

    fn select_previous_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id =
            (self.current_predicate_id + self.predicates.len() - 1) % self.predicates.len();
        self.update_display();
    }

    fn select_next_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id = (self.current_predicate_id + 1) % self.predicates.len();
        self.update_display();
    }

    fn get_selected_predicate(&self) -> Option<&Predicate> {
        self.predicates.get(self.current_predicate_id)
    }

    fn set_state(&mut self, state: bool) {
        let state = if state { "resumed" } else { "paused" };
        println!("state: {state}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_none() {
        use crate::frontend::Predicate;
        use crate::frontend::{Frontend, None};

        let mut none = None;
        none.set_input_text("hello");
        none.set_screen_size((64, 64));
        none.set_position((64.0, 64.0));
        none.set_input_text("input");
        none.set_max_predicates(10);
        none.add_predicate(Predicate::default());
        none.update_display();
        none.clear_all_predicates();
        none.select_previous_predicate();
        none.select_next_predicate();
        none.get_selected_predicate();
    }

    #[test]
    fn test_console() {
        use crate::frontend::{Console, Frontend, Predicate};

        let mut console = Console::default();
        console.set_max_predicates(10);
        console.set_screen_size((0, 0));
        console.set_position((0.0, 0.0));
        console.set_input_text("he");

        console.add_predicate(Predicate {
            code: "hell".to_owned(),
            remaining_code: "llo".to_owned(),
            texts: vec!["hello".to_owned()],
            can_commit: false,
        });
        console.add_predicate(Predicate {
            code: "helip".to_owned(),
            remaining_code: "lip".to_owned(),
            texts: vec![],
            can_commit: false,
        });
        console.add_predicate(Predicate {
            code: "helio".to_owned(),
            remaining_code: "s".to_owned(),
            texts: vec!["".to_owned()],
            can_commit: false,
        });
        console.add_predicate(Predicate {
            code: "heal".to_owned(),
            remaining_code: "al".to_owned(),
            texts: vec!["health".to_owned()],
            can_commit: false,
        });
        assert_eq!(console.predicates.len(), 2);
        console.update_display();
        console.select_previous_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&Predicate {
                code: "heal".to_owned(),
                remaining_code: "al".to_owned(),
                texts: vec!["health".to_owned()],
                can_commit: false
            })
        );
        console.select_next_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&Predicate {
                code: "hell".to_owned(),
                remaining_code: "llo".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: false
            })
        );

        console.clear_all_predicates();
        console.select_previous_predicate();
        console.select_next_predicate();
        assert!(console.get_selected_predicate().is_none());
    }
}
