use std::collections::HashMap;

pub struct Translator {
    dictionary: HashMap<String, String>,
    auto_commit: bool,
}

impl Translator {
    pub fn new(dictionary: HashMap<String, String>, auto_commit: bool) -> Self {
        Self {
            dictionary,
            auto_commit,
        }
    }

    pub fn translate(&self, input: &str) -> Vec<(String, String, String, bool)> {
        self.dictionary
            .iter()
            .filter_map(|(k, v)| {
                if k == input {
                    Some((k.to_owned(), "".to_owned(), v.to_owned(), self.auto_commit))
                } else if input.len() > 1 && k.starts_with(input) {
                    Some((
                        k.to_owned(),
                        k.chars().skip(input.len()).collect(),
                        v.to_owned(),
                        false,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}
