//! Engine to generate predicates based on a particular input.
//!
//! Example
//! ```rust
//! #[cfg(feature = "rhai")]
//! use afrim_translator::Engine;
//! use afrim_translator::Translator;
//! use std::collections::HashMap;
//!
//! // Translation via dictionary
//! let mut dictionary = HashMap::new();
//! dictionary.insert("halo".to_string(), ["hello".to_string()].to_vec());
//! dictionary.insert("nihao".to_string(), ["hello".to_string()].to_vec());
//!
//! // We build the translator.
//! let mut translator = Translator::new(dictionary, true);
//!
//! // Translation via scripting
//! #[cfg(feature = "rhai")]
//! {
//!     let engine = Engine::new();
//!     let hi = engine.compile(r#"
//!         fn translate(input) {
//!             if input == "hi" {
//!                 ["hi", "", "hello", true]
//!             }
//!         }
//!     "#).unwrap();
//!     translator.register("hi".to_string(), hi);
//! }
//!
//! #[cfg(feature = "rhai")]
//! assert_eq!(
//!     translator.translate("hi"),
//!     vec![(
//!         "hi".to_owned(),
//!         "".to_owned(),
//!         vec!["hello".to_owned()],
//!         true
//!     )]
//! );
//! ```
//!

#![deny(missing_docs)]

#[cfg(feature = "rhai")]
pub use rhai::Engine;
#[cfg(feature = "rhai")]
use rhai::{Array, Scope, AST};
use std::collections::HashMap;

/// Core structure of the translator.
pub struct Translator {
    dictionary: HashMap<String, Vec<String>>,
    #[cfg(feature = "rhai")]
    translators: HashMap<String, AST>,
    auto_commit: bool,
}

impl Translator {
    /// Initiate a new translator.
    pub fn new(dictionary: HashMap<String, Vec<String>>, auto_commit: bool) -> Self {
        Self {
            dictionary,
            auto_commit,
            #[cfg(feature = "rhai")]
            translators: HashMap::default(),
        }
    }

    #[cfg(feature = "rhai")]
    /// Register a translator
    pub fn register(&mut self, name: String, ast: AST) {
        self.translators.insert(name, ast);
    }

    #[cfg(feature = "rhai")]
    /// Unregister a translator
    pub fn unregister(&mut self, name: &str) {
        self.translators.remove(name);
    }

    /// Generate a list of predicates based on the input.
    pub fn translate(&self, input: &str) -> Vec<(String, String, Vec<String>, bool)> {
        #[cfg(feature = "rhai")]
        let mut scope = Scope::new();
        #[cfg(feature = "rhai")]
        let engine = Engine::new();

        let predicates = self.dictionary.iter().filter_map(|(key, value)| {
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
        });
        #[cfg(feature = "rhai")]
        let predicates =
            predicates.chain(self.translators.iter().filter_map(|(_name, translator)| {
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
            }));
        predicates.collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_translate() {
        #[cfg(feature = "rhai")]
        use crate::Engine;
        use crate::Translator;
        use std::collections::HashMap;

        // We build the translation
        let mut dictionary = HashMap::new();
        dictionary.insert("halo".to_string(), ["hello".to_string()].to_vec());

        // We config the translator
        #[cfg(not(feature = "rhai"))]
        let translator = Translator::new(dictionary, true);
        #[cfg(feature = "rhai")]
        let mut translator = Translator::new(dictionary, true);

        //
        #[cfg(feature = "rhai")]
        {
            let engine = Engine::new();
            let ast1 = engine.compile("fn translate(input) {}").unwrap();
            let ast2 = engine
                .compile(
                    r#"
                fn translate(input) {
                    if input == "hi" {
                        ["hi", "", "hello", true]
                    }
                }
            "#,
                )
                .unwrap();
            translator.register("none".to_string(), ast1);
            translator.unregister("none");
            translator.register("some".to_string(), ast2);
        }

        assert_eq!(translator.translate("h"), vec![]);
        #[cfg(feature = "rhai")]
        assert_eq!(
            translator.translate("hi"),
            vec![(
                "hi".to_owned(),
                "".to_owned(),
                vec!["hello".to_owned()],
                true
            )]
        );
        assert_eq!(
            translator.translate("ha"),
            vec![(
                "halo".to_owned(),
                "lo".to_owned(),
                vec!["hello".to_owned()],
                false
            )]
        );
        assert_eq!(
            translator.translate("halo"),
            vec![(
                "halo".to_owned(),
                "".to_owned(),
                vec!["hello".to_owned()],
                true
            )]
        );
    }
}
