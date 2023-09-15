use rhai::{Engine, AST};
use serde::Deserialize;
use std::result::Result;
use std::{collections::HashMap, error, fs, path::Path};
use toml::{self};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub core: Option<CoreConfig>,
    data: Option<HashMap<String, Data>>,
    translators: Option<HashMap<String, Data>>,
    translation: Option<HashMap<String, Data>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CoreConfig {
    pub buffer_size: Option<usize>,
    pub auto_capitalize: Option<bool>,
    pub page_size: Option<usize>,
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
    pub fn from_file(filepath: &Path) -> Result<Self, Box<dyn error::Error>> {
        let content = fs::read_to_string(filepath)
            .map_err(|err| format!("Couldn't open file `{filepath:?}`.\nCaused by:\n\t{err}."))?;
        let mut config: Self = toml::from_str(&content).map_err(|err| {
            format!("Failed to parse configuration file `{filepath:?}`.\nCaused by:\n\t{err}")
        })?;
        let config_path = filepath.parent().unwrap();
        let auto_capitalize = config
            .core
            .as_ref()
            .map(|c| c.auto_capitalize.unwrap_or(true))
            .unwrap_or(true);

        // Data
        let mut data = HashMap::new();

        config.data.unwrap_or_default().iter().try_for_each(
            |(key, value)| -> Result<(), Box<dyn error::Error>> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_file(&filepath)?;
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
                    _ => Err(format!("Invalid script file `{filepath:?}`.\nCaused by:\n\t{value:?} not allowed in the data table."))?,
                };
                Ok(())
            },
        )?;
        config.data = Some(data);

        // Translators
        let mut translators = HashMap::new();

        config.translators.unwrap_or_default().iter().try_for_each(
            |(key, value)| -> Result<(), Box<dyn error::Error>> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_file(&filepath)?;
                        translators.extend(conf.translators.unwrap_or_default());
                    }
                    Data::Simple(v) => {
                        let filepath = config_path.join(v.clone()).to_str().unwrap().to_string();
                        translators.insert(key.to_owned(), Data::Simple(filepath));
                    }
                    _ => Err(format!("Invalid script file `{filepath:?}`.\nCaused by:\n\t{value:?} not allowed in the translator table."))?,
                };
                Ok(())
            },
        )?;
        config.translators = Some(translators);

        // Translation
        let mut translation = HashMap::new();

        config.translation.unwrap_or_default().iter().try_for_each(
            |(key, value)| -> Result<(), Box<dyn error::Error>> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_file(&filepath)?;
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
                        alias.iter().chain([key.to_owned()].iter()).for_each(|e| {
                            translation.insert(e.to_owned(), Data::Multi(values.clone()));
                        });
                    }
                };
                Ok(())
            },
        )?;

        config.translation = Some(translation);

        Ok(config)
    }

    pub fn extract_data(&self) -> HashMap<String, String> {
        let empty = HashMap::default();

        self.data
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(k, v)| {
                let v = match v {
                    Data::Simple(value) => Some(value),
                    _ => None,
                };
                v.map(|v| (k.to_owned(), v.to_owned()))
            })
            .collect()
    }

    pub fn extract_translators(&self) -> Result<HashMap<String, AST>, Box<dyn error::Error>> {
        let empty = HashMap::default();
        let mut engine = Engine::new();

        // allow nesting up to 50 layers of expressions/statements
        // at global level, but only 10 inside function
        engine.set_max_expr_depths(25, 25);

        self.translators
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(name, filename)| {
                let filename = match filename {
                    Data::Simple(filename) => Some(filename),
                    _ => None,
                };

                filename.map(|filename| {
                    let parent = Path::new(&filename).parent().unwrap().to_str().unwrap();
                    let header = format!(r#"const DIR = {parent:?};"#);
                    let ast = engine.compile_file(filename.into()).map_err(|err| {
                        format!("Failed to parse script file `{filename}`.\nCaused by:\n\t{err}.")
                    })?;
                    let ast = engine.compile(header).unwrap().merge(&ast);

                    Ok((name.to_owned(), ast))
                })
            })
            .collect()
    }

    pub fn extract_translation(&self) -> HashMap<String, Vec<String>> {
        let empty = HashMap::new();

        self.translation
            .as_ref()
            .unwrap_or(&empty)
            .iter()
            .filter_map(|(k, v)| {
                let v = match v {
                    Data::Simple(v) => Some(vec![v.to_owned()]),
                    Data::Multi(v) => Some(v.to_owned()),
                    _ => None,
                };

                v.map(|v| (k.to_owned(), v))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_file() {
        use crate::config::Config;
        use std::path::Path;

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

        // parsing error
        let conf = Config::from_file(Path::new("./data/invalid_file.toml"));
        assert!(conf.is_err());

        // config file not found
        let conf = Config::from_file(Path::new("./data/not_found"));
        assert!(conf.is_err());

        // data and and core not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 0);

        // invalid data
        let conf = Config::from_file(Path::new("./data/invalid_data.toml"));
        assert!(conf.is_err());

        // invalid translator
        let conf = Config::from_file(Path::new("./data/invalid_translator.toml"));
        assert!(conf.is_err());
    }

    #[test]
    fn from_file_with_translators() {
        use crate::config::Config;
        use std::path::Path;

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
        use crate::config::Config;
        use std::path::Path;

        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 4);

        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let translation = conf.extract_translation();
        assert_eq!(translation.keys().len(), 0);
    }
}
