#![deny(missing_docs)]
//! Library to manage the configuration of the afrim input method.
//!
//! It's based on the top of the [`toml`](toml) crate.
//!
//! # Example
//!
//! ```no_run
//! use afrim_config::Config;
//! use std::path::Path;
//!
//! let filepath = Path::new("./data/config_sample.toml");
//! let conf = Config::from_file(&filepath).unwrap();
//!
//! # assert_eq!(conf.extract_data().keys().len(), 23);
//! # #[cfg(feature = "rhai")]
//! # assert_eq!(conf.extract_translators().unwrap().keys().len(), 2);
//! # assert_eq!(conf.extract_translation().keys().len(), 4);
//! ```
//!
//! In case that you want control the filesystem (reading of file), you can use the
//! [`Config::from_filesystem`](crate::Config::from_filesystem) method.
//!
//! # Example
//!
//! ```
//! use afrim_config::{Config, FileSystem};
//! use std::{error, path::Path, string::String};
//!
//! // Implements a custom filesystem.
//! struct File {
//!     source: String,
//! }
//!
//! impl File {
//!     pub fn new(source: String) -> Self {
//!         Self { source }
//!     }
//! }
//!
//! impl FileSystem for File {
//!     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
//!         Ok(self.source.to_string())
//!     }
//! }
//!
//! // Sets the config file.
//! let config_file = File::new(r#"
//! [core]
//! auto_commit = false
//!
//! [data]
//! "n*" = "ŋ"
//! "#.to_owned());
//!
//! // Loads the config file.
//! let config = Config::from_filesystem(&Path::new("."), &config_file).unwrap();
//!
//! assert_eq!(config.core.clone().unwrap().auto_commit, Some(false));
//! // Note that the auto_capitalize is enabled by default.
//! assert_eq!(
//!     Vec::from_iter(config.extract_data().into_iter()),
//!     vec![("n*".to_owned(), "ŋ".to_owned()), ("N*".to_owned(), "Ŋ".to_owned())]
//! );
//! ```

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
#[cfg(feature = "rhai")]
use rhai::{Engine, AST};
use serde::Deserialize;
use std::{fs, path::Path};
use toml::{self};

/// Trait to customize the filesystem.
pub trait FileSystem {
    /// Alternative to the fs::read_to_string.
    fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error>;
}

// Representation of the std::fs.
struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
        fs::read_to_string(filepath)
    }
}

/// Holds information about a configuration.
///
/// # Example
///
/// ```no_run
/// # use afrim_config::{Config, FileSystem};
/// # use std::{error, path::Path, string::String};
/// #
/// # // Implements a custom filesystem.
/// # struct File {
/// #     source: String,
/// # }
/// #
/// # impl File {
/// #     pub fn new(source: String) -> Self {
/// #         Self { source }
/// #     }
/// # }
/// #
/// # impl FileSystem for File {
/// #     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
/// #         Ok(self.source.to_string())
/// #     }
/// # }
/// #
/// # // Sets the config file.
/// # let config_file = File::new(r#"
/// [info]
/// description = "Sample Config File"
/// version = "2023-10-02"
///
/// [data]
/// 2a_ = "á̠"
/// ".?" = { value = "ʔ", alias = ["?."] }
/// emoji = { path = "./emoji.toml" }
///
/// [translation]
/// hey = "hi"
/// hi = { value = "hello", alias = ["hey"] }
/// hola = { values = ["hello"], alias = [] }
/// dictionary = { path = "./dictionary.toml" }
///
/// [translators]
/// date = "./scripts/datetime/date.rhai"
/// # "#.to_owned());
///
/// # // Loads the config file.
/// # Config::from_filesystem(&Path::new("."), &config_file).unwrap();
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// The core config.
    pub core: Option<CoreConfig>,
    data: Option<IndexMap<String, Data>>,
    #[cfg(feature = "rhai")]
    translators: Option<IndexMap<String, Data>>,
    translation: Option<IndexMap<String, Data>>,
}

/// Core information about a configuration.
///
/// # Example
///
/// ```
/// # use afrim_config::{Config, FileSystem};
/// # use std::{error, path::Path, string::String};
/// #
/// # // Implements a custom filesystem.
/// # struct File {
/// #     source: String,
/// # }
/// #
/// # impl File {
/// #     pub fn new(source: String) -> Self {
/// #         Self { source }
/// #     }
/// # }
/// #
/// # impl FileSystem for File {
/// #     fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
/// #         Ok(self.source.to_string())
/// #     }
/// # }
/// #
/// # // Sets the config file.
/// # let config_file = File::new(r#"
/// [core]
/// buffer_size = 32
/// auto_capitalize = false
/// page_size = 10
/// auto_commit = true
/// # "#.to_owned());
/// #
/// # // Loads the config file.
/// # Config::from_filesystem(&Path::new("."), &config_file).unwrap();
/// ```
#[derive(Deserialize, Debug, Clone)]
pub struct CoreConfig {
    /// The size of the memory (history).
    /// The number of elements that should be tracked.
    pub buffer_size: Option<usize>,
    auto_capitalize: Option<bool>,
    /// The max numbers of predicates to display.
    pub page_size: Option<usize>,
    /// Whether the predicate should be automatically committed.
    pub auto_commit: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Data {
    Simple(String),
    Multi(Vec<String>),
    File(DataFile),
    Detailed(DetailedData),
    MoreDetailed(MoreDetailedData),
}

#[derive(Deserialize, Debug, Clone)]
struct DataFile {
    path: String,
}

#[derive(Deserialize, Debug, Clone)]
struct DetailedData {
    value: String,
    alias: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct MoreDetailedData {
    values: Vec<String>,
    alias: Vec<String>,
}

macro_rules! insert_with_auto_capitalize {
    ( $data: expr, $auto_capitalize: expr, $key: expr, $value: expr ) => {
        $data.insert($key.to_owned(), Data::Simple($value.to_owned()));

        if $auto_capitalize && !$key.is_empty() && $key.chars().next().unwrap().is_lowercase() {
            $data
                .entry($key[0..1].to_uppercase() + &$key[1..])
                .or_insert(Data::Simple($value.to_uppercase()));
        }
    };
}

impl Config {
    /// Load the configuration from a file.
    pub fn from_file(filepath: &Path) -> Result<Self> {
        Self::from_filesystem(filepath, &StdFileSystem {})
    }

    /// Loads the configuration from a file in using a specified filesystem.
    pub fn from_filesystem(filepath: &Path, fs: &impl FileSystem) -> Result<Self> {
        let content = fs
            .read_to_string(filepath)
            .with_context(|| format!("Couldn't open file {filepath:?}."))?;
        let mut config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse configuration file {filepath:?}."))?;
        let config_path = filepath.parent().unwrap();
        let auto_capitalize = config
            .core
            .as_ref()
            .and_then(|c| c.auto_capitalize)
            .unwrap_or(true);

        // Data
        let mut data = IndexMap::new();

        config
            .data
            .unwrap_or_default()
            .iter()
            .try_for_each(|(key, value)| -> Result<()> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_filesystem(&filepath, fs)?;
                        data.extend(conf.data.unwrap_or_default());
                    }
                    Data::Simple(value) => {
                        insert_with_auto_capitalize!(data, auto_capitalize, key, value);
                    }
                    Data::Detailed(DetailedData { value, alias }) => {
                        alias.iter().chain([key.to_owned()].iter()).for_each(|key| {
                            insert_with_auto_capitalize!(data, auto_capitalize, key, value);
                        });
                    }
                    _ => Err(anyhow!("{value:?} not allowed in the data table."))
                        .with_context(|| format!("Invalid configuration file {filepath:?}."))?,
                };
                Ok(())
            })?;
        config.data = Some(data);

        // Translators
        #[cfg(feature = "rhai")]
        {
            let mut translators = IndexMap::new();

            config.translators.unwrap_or_default().iter().try_for_each(
                |(key, value)| -> Result<()> {
                    match value {
                        Data::File(DataFile { path }) => {
                            let filepath = config_path.join(path);
                            let conf = Config::from_filesystem(&filepath, fs)?;
                            translators.extend(conf.translators.unwrap_or_default());
                        }
                        Data::Simple(value) => {
                            let filepath = config_path
                                .join(value.clone())
                                .to_str()
                                .unwrap()
                                .to_string();
                            translators.insert(key.to_owned(), Data::Simple(filepath));
                        }
                        _ => Err(anyhow!("{value:?} not allowed in the translator table"))
                            .with_context(|| format!("Invalid configuration file {filepath:?}."))?,
                    };
                    Ok(())
                },
            )?;
            config.translators = Some(translators);
        }

        // Translation
        let mut translation = IndexMap::new();

        config.translation.unwrap_or_default().iter().try_for_each(
            |(key, value)| -> Result<()> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_filesystem(&filepath, fs)?;
                        translation.extend(conf.translation.unwrap_or_default());
                    }
                    Data::Simple(_) | Data::Multi(_) => {
                        translation.insert(key.to_owned(), value.clone());
                    }
                    Data::Detailed(DetailedData { value, alias }) => {
                        alias.iter().chain([key.to_owned()].iter()).for_each(|e| {
                            translation.insert(e.to_owned(), Data::Simple(value.to_owned()));
                        });
                    }
                    Data::MoreDetailed(MoreDetailedData { values, alias }) => {
                        alias.iter().chain([key.to_owned()].iter()).for_each(|key| {
                            translation.insert(key.to_owned(), Data::Multi(values.clone()));
                        });
                    }
                };
                Ok(())
            },
        )?;

        config.translation = Some(translation);

        Ok(config)
    }

    /// Extracts the data from the configuration.
    pub fn extract_data(&self) -> IndexMap<String, String> {
        let empty = IndexMap::default();

        self.data
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(key, value)| {
                let value = match value {
                    Data::Simple(value) => Some(value),
                    _ => None,
                };
                value.map(|value| (key.to_owned(), value.to_owned()))
            })
            .collect()
    }

    /// Extracts the translators from the configuration.
    #[cfg(feature = "rhai")]
    pub fn extract_translators(&self) -> Result<IndexMap<String, AST>> {
        self.extract_translators_using_filesystem(&StdFileSystem {})
    }

    /// Extracts the translators from the configuration using the specified filesystem..
    #[cfg(feature = "rhai")]
    pub fn extract_translators_using_filesystem(
        &self,
        fs: &impl FileSystem,
    ) -> Result<IndexMap<String, AST>> {
        let empty = IndexMap::default();
        let engine = Engine::new();

        self.translators
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(name, file_path)| {
                let file_path = match file_path {
                    Data::Simple(file_path) => Some(file_path),
                    _ => None,
                };

                file_path.map(|file_path| {
                    let file_path = Path::new(&file_path);
                    let parent = file_path.parent().unwrap().to_str().unwrap();
                    let header = format!(r#"const DIR = {parent:?};"#);
                    let body = fs
                        .read_to_string(file_path)
                        .with_context(|| format!("Couldn't open script file {file_path:?}."))?;
                    let ast = engine
                        .compile(body)
                        .with_context(|| format!("Failed to parse script file {file_path:?}."))?;
                    let ast = engine.compile(header).unwrap().merge(&ast);

                    Ok((name.to_owned(), ast))
                })
            })
            .collect()
    }

    /// Extracts the translation from the configuration.
    pub fn extract_translation(&self) -> IndexMap<String, Vec<String>> {
        let empty = IndexMap::new();

        self.translation
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(key, value)| {
                let value = match value {
                    Data::Simple(value) => Some(vec![value.to_owned()]),
                    Data::Multi(value) => Some(value.to_owned()),
                    _ => None,
                };

                value.map(|value| (key.to_owned(), value))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use std::path::Path;

    #[test]
    fn from_file() {
        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();

        assert_eq!(
            conf.core.as_ref().map(|core| {
                assert_eq!(core.buffer_size.unwrap(), 64);
                assert!(!core.auto_capitalize.unwrap());
                assert!(!core.auto_commit.unwrap());
                assert_eq!(core.page_size.unwrap(), 10);
                true
            }),
            Some(true)
        );

        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 23);

        // data and core not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 0);

        // parsing error
        let conf = Config::from_file(Path::new("./data/invalid_file.toml"));
        assert!(conf.is_err());

        // config file not found
        let conf = Config::from_file(Path::new("./data/not_found"));
        assert!(conf.is_err());
    }

    #[test]
    fn from_invalid_file() {
        // invalid data
        let conf = Config::from_file(Path::new("./data/invalid_data.toml"));
        assert!(conf.is_err());
    }

    #[cfg(feature = "rhai")]
    #[test]
    fn from_file_with_translators() {
        // invalid translator
        let conf = Config::from_file(Path::new("./data/invalid_translator.toml"));
        assert!(conf.is_err());

        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        let translators = conf.extract_translators().unwrap();
        assert_eq!(translators.keys().len(), 2);

        // translators not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let translators = conf.extract_translators().unwrap();
        assert_eq!(translators.keys().len(), 0);

        // scripts parsing error
        let conf = Config::from_file(Path::new("./data/bad_script2.toml")).unwrap();
        assert!(conf.extract_translators().is_err());

        // script file not found
        let conf = Config::from_file(Path::new("./data/bad_script.toml")).unwrap();
        assert!(conf.extract_translators().is_err());
    }

    #[test]
    fn from_file_with_translation() {
        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 4);

        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 0);
    }

    #[test]
    fn from_filesystem() {
        use crate::FileSystem;
        use std::fs;

        #[derive(Default)]
        struct FilterFileSystem;

        impl FileSystem for FilterFileSystem {
            fn read_to_string(&self, filepath: &Path) -> Result<String, std::io::Error> {
                let file_stem = filepath.file_stem().unwrap();

                Ok(if file_stem == "data_sample" {
                    fs::read_to_string(filepath)?
                } else {
                    String::new()
                })
            }
        }

        let fs = FilterFileSystem {};
        let filepath = Path::new("./data/data_sample.toml");
        let conf = Config::from_filesystem(filepath, &fs).unwrap();

        assert_eq!(conf.extract_data().keys().len(), 13);
        #[cfg(feature = "rhai")]
        assert_eq!(conf.extract_translators().unwrap().keys().len(), 0);
        assert_eq!(conf.extract_translation().keys().len(), 0);
    }
}
