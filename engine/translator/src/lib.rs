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
//! use afrim_translator::Translator;
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
//!         (
//!             "jump".to_owned(),
//!             "".to_owned(),
//!             vec!["sauter".to_owned()],
//!             true
//!         ),
//!         // Auto-completion.
//!         (
//!             "jumper".to_owned(),
//!             "er".to_owned(),
//!             vec!["sauteur".to_owned()],
//!             false
//!         )
//!     ]
//! );
//! ```
//!
//! # Example with the strsim feature
//!
//! ```
//! use afrim_translator::Translator;
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
//!     vec![(
//!         "jump".to_owned(),
//!         "".to_owned(),
//!         vec!["sauter".to_owned()],
//!         false
//!     )]
//! );
//! ```
//!
//! # Example with the rhai feature
//!
//! ```
//! #[cfg(feature = "rhai")]
//! use afrim_translator::Engine;
//! use afrim_translator::Translator;
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
//!         (
//!             "jump".to_owned(),
//!             "".to_owned(),
//!             vec!["sauter".to_owned()],
//!             true
//!         ),
//!         #[cfg(feature = "rhai")]
//!         // Programmable translation.
//!         (
//!             "jump".to_owned(),
//!             "".to_owned(),
//!             vec!["\n".to_owned()],
//!             false
//!         ),
//!         // Auto-completion.
//!         (
//!             "jumper".to_owned(),
//!             "er".to_owned(),
//!             vec!["sauteur".to_owned()],
//!             false
//!         )
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

type P = (String, String, Vec<String>, bool);

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
    /// use afrim_translator::{Engine, Translator};
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
    ///         ("09/02/2024".to_owned(), "".to_owned(),
    ///         vec!["9, Feb 2024".to_owned()], true)
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
    /// use afrim_translator::{Engine, Translator};
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
    /// assert_eq!(translator.translate("hello"), vec![("hello".to_owned(), "".to_owned(), vec![], true)]);
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
    /// use afrim_translator::Translator;
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
    ///         (
    ///             "salut!".to_owned(), "ut!".to_owned(),
    ///             vec!["hello!".to_owned(), "hi!".to_owned()],
    ///             false
    ///         ),
    ///         (
    ///             "salade".to_owned(), "ade".to_owned(),
    ///             vec!["vegetable".to_owned()],
    ///             false
    ///         )
    ///     ]
    /// )
    /// ```
    pub fn translate(&self, input: &str) -> Vec<P> {
        #[cfg(feature = "rhai")]
        let mut scope = Scope::new();
        #[cfg(feature = "rhai")]
        let engine = Engine::new();
        let predicates = self.dictionary.iter().filter_map(|(key, value)| {
            if input.len() < 2 || input.len() > key.len() || key[0..1] != input[0..1] {
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
                let mut data = engine
                    .call_fn::<Array>(&mut scope, translator, "translate", (input.to_owned(),))
                    .unwrap_or_default();

                (data.len() == 4).then(|| {
                    let code = data.remove(0).into_string().unwrap();
                    let remaining_code = data.remove(0).into_string().unwrap();
                    let texts = data.remove(0);
                    let texts = if texts.is_array() {
                        texts.into_array().unwrap()
                    } else {
                        vec![texts]
                    };
                    let texts = texts
                        .into_iter()
                        .map(|e| e.into_string().unwrap())
                        .collect();
                    let translated = data.remove(0).as_bool().unwrap();

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
