#![deny(missing_docs)]
//! API to develop a frontend interface for the afrim.
//!

pub use afrim_translator::Predicate;

/// Trait that every afrim frontend should implement.
pub trait Frontend {
    /// Updates the frontend screen size.
    fn update_screen(&mut self, _screen: (u64, u64)) {}
    /// Updates the frontend position.
    fn update_position(&mut self, _position: (f64, f64)) {}
    /// Sets the current sequential code to display.
    fn set_input(&mut self, _text: &str) {}
    /// Sets the maximun number of predicates to be display.
    fn set_page_size(&mut self, _size: usize) {}
    /// Adds a predicate in the list of predicates.
    fn add_predicate(&mut self, _predicate: Predicate) {}
    /// Refreshs the display.
    fn display(&self) {}
    /// Clears the list of predicates.
    fn clear_predicates(&mut self) {}
    /// Selects the previous predicate.
    fn previous_predicate(&mut self) {}
    /// Selects the next predicate.
    fn next_predicate(&mut self) {}
    /// Returns the selected predicate.
    fn get_selected_predicate(&self) -> Option<&Predicate> {
        Option::None
    }
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
    fn set_page_size(&mut self, size: usize) {
        self.page_size = size;
        self.predicates = Vec::with_capacity(size);
    }

    fn set_input(&mut self, text: &str) {
        self.input = text.to_owned();
    }

    fn display(&self) {
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

    fn clear_predicates(&mut self) {
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

    fn previous_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id =
            (self.current_predicate_id + self.predicates.len() - 1) % self.predicates.len();
        self.display();
    }

    fn next_predicate(&mut self) {
        if self.predicates.is_empty() {
            return;
        };

        self.current_predicate_id = (self.current_predicate_id + 1) % self.predicates.len();
        self.display();
    }

    fn get_selected_predicate(&self) -> Option<&Predicate> {
        self.predicates.get(self.current_predicate_id)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_none() {
        use crate::frontend::Predicate;
        use crate::frontend::{Frontend, None};

        let mut none = None;
        none.set_input("hello");
        none.update_screen((64, 64));
        none.update_position((64.0, 64.0));
        none.set_input("input");
        none.set_page_size(10);
        none.add_predicate(Predicate::default());
        none.display();
        none.clear_predicates();
        none.previous_predicate();
        none.next_predicate();
        none.get_selected_predicate();
    }

    #[test]
    fn test_console() {
        use crate::frontend::{Console, Frontend, Predicate};

        let mut console = Console::default();
        console.set_page_size(10);
        console.update_screen((0, 0));
        console.update_position((0.0, 0.0));
        console.set_input("he");

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
        console.display();
        console.previous_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&Predicate {
                code: "heal".to_owned(),
                remaining_code: "al".to_owned(),
                texts: vec!["health".to_owned()],
                can_commit: false
            })
        );
        console.next_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&Predicate {
                code: "hell".to_owned(),
                remaining_code: "llo".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: false
            })
        );

        console.clear_predicates();
        console.previous_predicate();
        console.next_predicate();
        assert!(console.get_selected_predicate().is_none());
    }
}
