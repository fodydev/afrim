#![deny(missing_docs)]
//! This crate provides a range of language-related functionalities, including translation,
//! auto-suggestions, auto-correction and more.
//! It's designed to enhance the language processing tasks within in input method engine.
//!
//! **Note**: We use [`IndexMap`] instead of [`HashMap`](std::collections::HashMap) for better performance
//! when dealing with big datasets.
//!
//! ### Feature flags
//!
//! To reduce the amount of compiled code in the crate, you can enable feature manually. This is
//! done by adding `default-features = false` to your dependency specification. Below is a list of
//! the features available in this crate.
//!
//! * `rhai`: Enables the usage of rhai script files.
//! * `rhai-wasm`: Like rhai, but wasm compatible.
//! * `strsim`: Enables the text similarity algorithm for better predictions.
//! * `serde`: Enables serde feature.
//!
//! # Example
//!
//! ```
//! use afrim_translator::{Predicate, Translator};
//! use indexmap::IndexMap;
//!
//! // Prepares the dictionary.
//! let mut dictionary = IndexMap::new();
//! dictionary.insert("jump".to_string(), vec!["sauter".to_string()]);
//! dictionary.insert("jumper".to_string(), vec!["sauteur".to_string()]);
//! dictionary.insert("nihao".to_string(), vec!["hello".to_string()]);
//!
//! // Builds the translator.
//! let mut translator = Translator::new(dictionary, true);
//!
//! assert_eq!(
//!     translator.translate("jump"),
//!     vec![
//!         Predicate {
//!             code: "jump".to_owned(),
//!             remaining_code: "".to_owned(),
//!             texts: vec!["sauter".to_owned()],
//!             can_commit: true
//!         },
//!         // Auto-completion.
//!         Predicate {
//!             code: "jumper".to_owned(),
//!             remaining_code: "er".to_owned(),
//!             texts: vec!["sauteur".to_owned()],
//!             can_commit: false
//!         }
//!     ]
//! );
//! ```
//!
//! # Example with the strsim feature
//!
//! ```
//! use afrim_translator::{Predicate, Translator};
//! use indexmap::IndexMap;
//!
//! // Prepares the dictionary.
//! let mut dictionary = IndexMap::new();
//! dictionary.insert("jump".to_string(), vec!["sauter".to_string()]);
//! dictionary.insert("jumper".to_string(), vec!["sauteur".to_string()]);
//!
//! // Builds the translator.
//! let mut translator = Translator::new(dictionary, true);
//!
//! // Auto-suggestion / Auto-correction.
//! #[cfg(feature = "strsim")]
//! assert_eq!(
//!     translator.translate("junp"),
//!     vec![Predicate {
//!         code: "jump".to_owned(),
//!         remaining_code: "".to_owned(),
//!         texts: vec!["sauter".to_owned()],
//!         can_commit: false
//!     }]
//! );
//! ```
//!
//! # Example with the rhai feature
//!
//! ```
//! #[cfg(feature = "rhai")]
//! use afrim_translator::Engine;
//! use afrim_translator::{Translator, Predicate};
//! use indexmap::IndexMap;
//!
//! // Prepares the dictionary.
//! let mut dictionary = IndexMap::new();
//! dictionary.insert("jump".to_string(), vec!["sauter".to_string()]);
//! dictionary.insert("jumper".to_string(), vec!["sauteur".to_string()]);
//!
//! // Prepares the script.
//! #[cfg(feature = "rhai")]
//! let engine = Engine::new();
//! #[cfg(feature = "rhai")]
//! let jump_translator = engine.compile(r#"
//!     // The main script function.
//!     fn translate(input) {
//!         if input == "jump" {
//!             [input, "", "\n", false]
//!         }
//!     }
//! "#).unwrap();
//!
//! // Builds the translator.
//! let mut translator = Translator::new(dictionary, true);
//!
//! // Registers the jump translator.
//! #[cfg(feature = "rhai")]
//! translator.register("jump".to_string(), jump_translator);
//!
//! assert_eq!(
//!     translator.translate("jump"),
//!     vec![
//!         Predicate {
//!             code: "jump".to_owned(),
//!             remaining_code: "".to_owned(),
//!             texts: vec!["sauter".to_owned()],
//!             can_commit: true
//!         },
//!         #[cfg(feature = "rhai")]
//!         // Programmable translation.
//!         Predicate {
//!             code: "jump".to_owned(),
//!             remaining_code: "".to_owned(),
//!             texts: vec!["\n".to_owned()],
//!             can_commit: false
//!         },
//!         // Auto-completion.
//!         Predicate {
//!             code: "jumper".to_owned(),
//!             remaining_code: "er".to_owned(),
//!             texts: vec!["sauteur".to_owned()],
//!             can_commit: false
//!         }
//!     ]
//! );
//! ```

use indexmap::IndexMap;
#[cfg(feature = "rhai")]
pub use rhai::Engine;
#[cfg(feature = "rhai")]
use rhai::{Array, Scope, AST};
use std::cmp::Ordering;
#[cfg(feature = "strsim")]
use strsim::{self};

/// Struct representing the predicate.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Predicate {
    /// The predicate code.
    pub code: String,
    /// The remaining code to match the predicate.
    pub remaining_code: String,
    /// The resulting predicate possible outputs.
    pub texts: Vec<String>,
    /// Whether the predicate can be commit.
    pub can_commit: bool,
}

/// Core structure of the translator.
pub struct Translator {
    dictionary: IndexMap<String, Vec<String>>,
    #[cfg(feature = "rhai")]
    translators: IndexMap<String, AST>,
    auto_commit: bool,
}

impl Translator {
    /// Initiatializes a new translator.
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_translator::Translator;
    /// use indexmap::IndexMap;
    ///
    /// let dictionary = IndexMap::new();
    /// let translator = Translator::new(dictionary, false);
    /// ```
    pub fn new(dictionary: IndexMap<String, Vec<String>>, auto_commit: bool) -> Self {
        Self {
            dictionary,
            auto_commit,
            #[cfg(feature = "rhai")]
            translators: IndexMap::default(),
        }
    }

    #[cfg(feature = "rhai")]
    /// Registers a translator.
    ///
    /// The provided name will be used for debugging in case of script error.
    /// Note that the scripts are compiled using [`Engine`](crate::Engine::compile).
    ///
    /// # Example
    ///
    /// ```
    /// use afrim_translator::{Engine, Predicate, Translator};
    /// use indexmap::IndexMap;
    ///
    /// // We prepare the script.
    /// let date_translator = r#"
    ///    // Date converter.
    ///    
    ///    const MONTHS = [
    ///        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    ///        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
    ///    ];
    ///    
    ///    fn parse_date(input) {
    ///        let data = input.split('/');
    ///    
    ///        if data.len() != 3 {
    ///            return [];
    ///        }
    ///    
    ///        let day = parse_int(data[0]);
    ///        let month = parse_int(data[1]);
    ///        let year = parse_int(data[2]);
    ///    
    ///        if day in 1..31 && month in 1..13 && year in 1..2100 {
    ///            return [day, month, year];
    ///        }
    ///    }
    ///    
    ///    // Script main function.
    ///    fn translate(input) {
    ///        let date = parse_date(input);
    ///    
    ///        if date.is_empty() { return }
    ///    
    ///        let month = global::MONTHS[date[1]-1];
    ///    
    ///        [input, "", [`${date[0]}, ${month} ${date[2]}`], true]
    ///    }
    /// "#;
    /// let mut engine = Engine::new();
    /// let date_translator = engine.compile(date_translator).unwrap();
    ///
    /// // We build the translator.
    /// let mut translator = Translator::new(IndexMap::new(), true);
    ///
    /// // We register our date translator.
    /// translator.register("date_translator".to_owned(), date_translator);
    ///
    /// assert_eq!(
    ///     translator.translate("09/02/2024"),
    ///     vec![
    ///         Predicate {
    ///             code: "09/02/2024".to_owned(),
    ///             remaining_code: "".to_owned(),
    ///             texts: vec!["9, Feb 2024".to_owned()],
    ///             can_commit: true
    ///         }
    ///     ]
    /// );
    /// ```
    pub fn register(&mut self, name: String, ast: AST) {
        self.translators.insert(name, ast);
    }

    #[cfg(feature = "rhai")]
    /// Unregisters a translator.
    ///
    /// # Example
    /// ```
    /// use afrim_translator::{Engine, Predicate, Translator};
    /// use indexmap::IndexMap;
    ///
    /// // We prepare the script.
    /// let engine = Engine::new();
    /// let erase_translator = engine.compile("fn translate(input) { [input, \"\", [], true] }").unwrap();
    ///
    /// // We build the translator.
    /// let mut translator = Translator::new(IndexMap::new(), false);
    ///
    /// // We register the erase translator.
    /// translator.register("erase".to_owned(), erase_translator);
    /// assert_eq!(
    ///     translator.translate("hello"),
    ///     vec![
    ///         Predicate {
    ///             code: "hello".to_owned(),
    ///             remaining_code: "".to_owned(),
    ///             texts: vec![],
    ///             can_commit: true
    ///         }
    ///     ]
    /// );
    ///
    /// // We unregister the erase translator.
    /// translator.unregister("erase");
    /// assert_eq!(translator.translate("hello"), vec![]);
    /// ```
    pub fn unregister(&mut self, name: &str) {
        self.translators.shift_remove(name);
    }

    /// Generates a list of predicates based on the input.
    ///
    /// # Example
    ///
    /// ```
    /// use indexmap::IndexMap;
    /// use afrim_translator::{Predicate, Translator};
    ///
    /// // We prepares the dictionary.
    /// let mut dictionary = IndexMap::new();
    /// dictionary.insert("salut!".to_owned(), vec!["hello!".to_owned(), "hi!".to_owned()]);
    /// dictionary.insert("salade".to_owned(), vec!["vegetable".to_owned()]);
    ///
    /// // We build the translator.
    /// let translator = Translator::new(dictionary, false);
    /// assert_eq!(
    ///     translator.translate("sal"),
    ///     vec![
    ///         Predicate {
    ///             code: "salut!".to_owned(),
    ///             remaining_code: "ut!".to_owned(),
    ///             texts: vec!["hello!".to_owned(), "hi!".to_owned()],
    ///             can_commit: false
    ///         },
    ///         Predicate {
    ///             code: "salade".to_owned(),
    ///             remaining_code: "ade".to_owned(),
    ///             texts: vec!["vegetable".to_owned()],
    ///             can_commit: false
    ///         }
    ///     ]
    /// )
    /// ```
    pub fn translate(&self, input: &str) -> Vec<Predicate> {
        #[cfg(feature = "rhai")]
        let mut scope = Scope::new();
        #[cfg(feature = "rhai")]
        let engine = Engine::new();
        let predicates = self.dictionary.iter().filter_map(|(key, values)| {
            if input.chars().count() < 2 || input.len() > key.len() || key[0..1] != input[0..1] {
                return None;
            };

            let predicate = (key == input).then_some((
                1.0,
                Predicate {
                    code: key.to_owned(),
                    remaining_code: "".to_owned(),
                    texts: values.to_owned(),
                    can_commit: self.auto_commit,
                },
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
                            Predicate {
                                code: key.to_owned(),
                                remaining_code: "".to_owned(),
                                texts: values.to_owned(),
                                can_commit: false,
                            },
                        )
                    })
                } else {
                    None
                }
            });
            predicate.or_else(|| {
                key.starts_with(input).then_some((
                    0.5,
                    Predicate {
                        code: key.to_owned(),
                        remaining_code: key.chars().skip(input.len()).collect(),
                        texts: values.to_owned(),
                        can_commit: false,
                    },
                ))
            })
        });
        #[cfg(feature = "rhai")]
        let predicates =
            predicates.chain(self.translators.iter().filter_map(|(_name, translator)| {
                let mut data = engine
                    .call_fn::<Array>(&mut scope, translator, "translate", (input.to_owned(),))
                    .unwrap_or_default();

                (data.len() == 4).then(|| {
                    let code = data.remove(0).into_string().unwrap();
                    let remaining_code = data.remove(0).into_string().unwrap();
                    let value = data.remove(0);
                    let values = if value.is_array() {
                        value.into_array().unwrap()
                    } else {
                        vec![value]
                    };
                    let values = values
                        .into_iter()
                        .map(|e| e.into_string().unwrap())
                        .collect();
                    let translated = data.remove(0).as_bool().unwrap();

                    (
                        1.0,
                        Predicate {
                            code,
                            remaining_code,
                            texts: values,
                            can_commit: translated,
                        },
                    )
                })
            }));
        let mut predicates = predicates.collect::<Vec<(f64, Predicate)>>();

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
        use crate::{Predicate, Translator};
        use indexmap::IndexMap;

        // We build the translation
        let mut dictionary = IndexMap::new();
        dictionary.insert("halo".to_string(), ["hello".to_string()].to_vec());

        // We config the translator
        #[cfg(not(feature = "rhai"))]
        let translator = Translator::new(dictionary, true);
        #[cfg(feature = "rhai")]
        let mut translator = Translator::new(dictionary, true);

        // Test the filtering
        translator.translate("ù");
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
            vec![Predicate {
                code: "hi".to_owned(),
                remaining_code: "".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: true
            }]
        );
        assert_eq!(
            translator.translate("ha"),
            vec![Predicate {
                code: "halo".to_owned(),
                remaining_code: "lo".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: false
            }]
        );
        #[cfg(feature = "strsim")]
        assert_eq!(
            translator.translate("helo"),
            vec![Predicate {
                code: "halo".to_owned(),
                remaining_code: "".to_owned(),
                texts: vec!["hello".to_owned()],
                can_commit: false
            }]
        );
    }
}
