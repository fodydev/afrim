pub trait Frontend {
    fn update_screen(&mut self, _screen: (u64, u64)) {}
    fn update_position(&mut self, _position: (f64, f64)) {}
    fn set_input(&mut self, _text: &str) {}
    fn set_page_size(&mut self, _size: usize) {}
    fn add_predicate(&mut self, _code: &str, _remaining_code: &str, _text: &str) {}
    fn display(&self) {}
    fn clear_predicates(&mut self) {}
    fn previous_predicate(&mut self) {}
    fn next_predicate(&mut self) {}
    fn get_selected_predicate(&self) -> Option<&(String, String, String)> {
        Option::None
    }
}

pub struct None;

impl Frontend for None {}

#[derive(Default)]
pub struct Console {
    page_size: usize,
    predicates: Vec<(String, String, String)>,
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
                .map(|(id, (_code, remaining_code, text))| format!(
                    "{}{}. {} ~{}\t ",
                    if id == self.current_predicate_id {
                        "*"
                    } else {
                        ""
                    },
                    id + 1,
                    text,
                    remaining_code
                ))
                .collect::<Vec<_>>()
                .join("\t")
        );
    }

    fn clear_predicates(&mut self) {
        self.predicates.clear();
        self.current_predicate_id = 0;
    }

    fn add_predicate(&mut self, code: &str, remaining_code: &str, text: &str) {
        self.predicates
            .push((code.to_owned(), remaining_code.to_owned(), text.to_owned()));
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

    fn get_selected_predicate(&self) -> Option<&(String, String, String)> {
        self.predicates.get(self.current_predicate_id)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_none() {
        use crate::api::{Frontend, None};

        let mut none = None;
        none.set_input("hello");
        none.update_screen((64, 64));
        none.update_position((64.0, 64.0));
        none.set_input("input");
        none.set_page_size(10);
        none.add_predicate("hey", "y", "hello");
        none.display();
        none.clear_predicates();
        none.previous_predicate();
        none.next_predicate();
        none.get_selected_predicate();
    }

    #[test]
    fn test_console() {
        use crate::api::{Console, Frontend};

        let mut console = Console::default();
        console.set_page_size(10);
        console.update_screen((0, 0));
        console.update_position((0.0, 0.0));
        console.set_input("he");

        console.add_predicate("hell", "llo", "hello");
        console.add_predicate("helip", "lip", "helicopter");
        console.add_predicate("heal", "al", "health");
        console.display();
        console.previous_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&("heal".to_owned(), "al".to_owned(), "health".to_owned()))
        );
        console.next_predicate();
        assert_eq!(
            console.get_selected_predicate(),
            Some(&("hell".to_owned(), "llo".to_owned(), "hello".to_owned()))
        );

        console.clear_predicates();
        console.previous_predicate();
        console.next_predicate();
        assert!(console.get_selected_predicate().is_none());
    }
}
