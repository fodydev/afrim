//! Engine to generate predicates based on a particular input.
//!
//! Example
//! ```rust
//! #[cfg(feature = "rhai")]
//! use afrim_translator::Engine;
//! use afrim_translator::Translator;
//! use indexmap::IndexMap;
//!
//! // Translation via dictionary
//! let mut dictionary = IndexMap::new();
//! dictionary.insert("jump".to_string(), ["sauter".to_string()].to_vec());
//! dictionary.insert("jumper".to_string(), ["sauteur".to_string()].to_vec());
//! dictionary.insert("nihao".to_string(), ["hello".to_string()].to_vec());
//!
//! // We build the translator.
//! let mut translator = Translator::new(dictionary, true);
//!
//! // Translation via scripting
//! #[cfg(feature = "rhai")]
//! {
//!     let engine = Engine::new();
//!     let jump = engine.compile(r#"
//!         fn translate(input) {
//!             if input == "jump" {
//!                 [input, "", "\n", false]
//!             }
//!         }
//!     "#).unwrap();
//!     translator.register("jump".to_string(), jump);
//! }
//!
//! assert_eq!(
//!     translator.translate("jump"),
//!     vec![
//!         (
//!             "jump".to_owned(),
//!             "".to_owned(),
//!             vec!["sauter".to_owned()],
//!             true
//!         ),
//!         #[cfg(feature = "rhai")]
//!         // Programmable translation
//!         (
//!             "jump".to_owned(),
//!             "".to_owned(),
//!             vec!["\n".to_owned()],
//!             false
//!         ),
//!         // Auto-completion
//!         (
//!             "jumper".to_owned(),
//!             "er".to_owned(),
//!             vec!["sauteur".to_owned()],
//!             false
//!         )
//!     ]
//! );
//!
//! // Auto-suggestion / Auto-correction
//! #[cfg(feature = "strsim")]
//! assert_eq!(
//!     translator.translate("junp"),
//!     vec![(
//!         "jump".to_owned(),
//!         "".to_owned(),
//!         vec!["sauter".to_owned()],
//!         false
//!     )]
//! );
//! ```
//!

#![deny(missing_docs)]

use indexmap::IndexMap;
#[cfg(feature = "rhai")]
pub use rhai::Engine;
#[cfg(feature = "rhai")]
use rhai::{Array, Scope, AST};
use std::cmp::Ordering;
#[cfg(feature = "strsim")]
use strsim::{self};

type P = (String, String, Vec<String>, bool);

/// Core structure of the translator.
pub struct Translator {
    dictionary: IndexMap<String, Vec<String>>,
    #[cfg(feature = "rhai")]
    translators: IndexMap<String, AST>,
    auto_commit: bool,
}

impl Translator {
    /// Initiate a new translator.
    pub fn new(dictionary: IndexMap<String, Vec<String>>, auto_commit: bool) -> Self {
        Self {
            dictionary,
            auto_commit,
            #[cfg(feature = "rhai")]
            translators: IndexMap::default(),
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
    pub fn translate(&self, input: &str) -> Vec<P> {
        #[cfg(feature = "rhai")]
        let mut scope = Scope::new();
        #[cfg(feature = "rhai")]
        let engine = Engine::new();
        let predicates = self.dictionary.iter().filter_map(|(key, value)| {
            if input.len() < 2 || input.len() > key.len() {
                return None;
            };

            let predicate = (key == input).then_some((
                1.0,
                (
                    key.to_owned(),
                    "".to_owned(),
                    value.to_owned(),
                    self.auto_commit,
                ),
            ));
            #[cfg(feature = "strsim")]
            let predicate = predicate.or_else(|| {
                if key.len() == input.len() {
                    let confidence = strsim::hamming(key.as_ref(), input)
                        .map(|n| 1.0 - (n as f64 / key.len() as f64))
                        .unwrap_or(0.0);

                    (confidence > 0.7).then(|| {
                        (
                            confidence,
                            (key.to_owned(), "".to_owned(), value.to_owned(), false),
                        )
                    })
                } else {
                    None
                }
            });
            predicate.or_else(|| {
                key.starts_with(input).then_some((
                    0.5,
                    (
                        key.to_owned(),
                        key.chars().skip(input.len()).collect(),
                        value.to_owned(),
                        false,
                    ),
                ))
            })
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

                    (1.0, (code, remaining_code, texts, translated))
                })
            }));
        let mut predicates = predicates.collect::<Vec<(f64, P)>>();

        // from the best to the worst
        predicates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

        predicates
            .into_iter()
            .map(|(_, predicate)| predicate)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_translate() {
        #[cfg(feature = "rhai")]
        use crate::Engine;
        use crate::Translator;
        use indexmap::IndexMap;

        // We build the translation
        let mut dictionary = IndexMap::new();
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
        #[cfg(feature = "strsim")]
        assert_eq!(
            translator.translate("helo"),
            vec![(
                "halo".to_owned(),
                "".to_owned(),
                vec!["hello".to_owned()],
                false
            )]
        );
    }
}
