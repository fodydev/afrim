use std::collections::HashMap;

pub struct Translator {
    dictionary: HashMap<String, String>,
}

impl Translator {
    pub fn new(dictionary: HashMap<String, String>) -> Self {
        Self { dictionary }
    }

    pub fn translate(&self, input: &str) -> Vec<(String, String, bool)> {
        self.dictionary
            .iter()
            .filter_map(|(k, v)| {
                if k == input {
                    Some((k.to_owned(), v.to_owned(), true))
                } else if input.len() > 2 && k.starts_with(input) {
                    Some((k.chars().skip(input.len()).collect(), v.to_owned(), false))
                } else {
                    None
                }
            })
            .collect()
    }
}
