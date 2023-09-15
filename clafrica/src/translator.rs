use rhai::{Array, Engine, Scope, AST};
use std::collections::HashMap;

pub struct Translator {
    dictionary: HashMap<String, Vec<String>>,
    translators: HashMap<String, AST>,
    auto_commit: bool,
}

impl Translator {
    pub fn new(
        dictionary: HashMap<String, Vec<String>>,
        translators: HashMap<String, AST>,
        auto_commit: bool,
    ) -> Self {
        Self {
            dictionary,
            translators,
            auto_commit,
        }
    }

    pub fn translate(&self, input: &str) -> Vec<(String, String, Vec<String>, bool)> {
        let mut scope = Scope::new();
        let engine = Engine::new();

        self.dictionary
            .iter()
            .filter_map(|(key, value)| {
                if key == input {
                    Some((
                        key.to_owned(),
                        "".to_owned(),
                        value.to_owned(),
                        self.auto_commit,
                    ))
                } else if input.len() > 1 && key.starts_with(input) {
                    Some((
                        key.to_owned(),
                        key.chars().skip(input.len()).collect(),
                        value.to_owned(),
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
                    let texts = data[2]
                        .clone()
                        .into_array()
                        .unwrap_or(vec![data[2].clone()])
                        .iter()
                        .map(|e| e.clone().into_string().unwrap())
                        .collect();
                    let translated = data[3].clone().as_bool().unwrap();

                    (code, remaining_code, texts, translated)
                })
            }))
            .collect()
    }
}
