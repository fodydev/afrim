use rhai::{Array, Engine, Scope, AST};
use std::collections::HashMap;

pub struct Translator {
    dictionary: HashMap<String, String>,
    translators: HashMap<String, AST>,
    auto_commit: bool,
}

impl Translator {
    pub fn new(
        dictionary: HashMap<String, String>,
        translators: HashMap<String, AST>,
        auto_commit: bool,
    ) -> Self {
        Self {
            dictionary,
            translators,
            auto_commit,
        }
    }

    pub fn translate(&self, input: &str) -> Vec<(String, String, String, bool)> {
        let mut scope = Scope::new();
        let engine = Engine::new();

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
            .chain(self.translators.iter().filter_map(|(_name, translator)| {
                let data = engine
                    .call_fn::<Array>(&mut scope, translator, "translate", (input.to_owned(),))
                    .unwrap_or_default();

                (data.len() == 4).then(|| {
                    let code = data[0].clone().into_string().unwrap();
                    let remaining_code = data[1].clone().into_string().unwrap();
                    let text = data[2].clone().into_string().unwrap();
                    let translated = data[3].clone().as_bool().unwrap();

                    (code, remaining_code, text, translated)
                })
            }))
            .collect()
    }
}
