use serde::Deserialize;
use std::{collections::HashMap, error, fs, path::Path};
use toml::{self};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub core: Option<CoreConfig>,
    data: HashMap<String, Data>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CoreConfig {
    pub buffer_size: usize,
    pub auto_capitalize: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Data {
    Simple(String),
    File(DataFile),
    Detailed(DetailedData),
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

impl Config {
    pub fn from_file(filepath: &Path) -> Result<Self, Box<dyn error::Error>> {
        let content = fs::read_to_string(filepath)
            .map_err(|err| format!("Couldn't open file `{filepath:?}`.\nCaused by:\n\t{err}."))?;
        let mut config: Self = toml::from_str(&content).map_err(|err| {
            format!("Failed to parse configuration file `{filepath:?}`.\nCaused by:\n\t{err}")
        })?;

        let config_path = filepath.parent().unwrap();

        let mut data = HashMap::new();

        config
            .data
            .iter()
            .try_for_each(|(key, value)| -> Result<(), Box<dyn error::Error>> {
                match value {
                    Data::File(DataFile { path }) => {
                        let filepath = config_path.join(path);
                        let conf = Config::from_file(&filepath)?;
                        data.extend(conf.data);
                    }
                    Data::Simple(_) => {
                        data.insert(key.to_owned(), value.clone());
                    }
                    Data::Detailed(DetailedData { value, alias }) => {
                        alias.iter().chain([key.to_owned()].iter()).for_each(|e| {
                            data.insert(e.to_owned(), Data::Simple(value.to_owned()));
                        });
                    }
                };
                Ok(())
            })?;

        config.data = data;

        Ok(config)
    }

    pub fn extract_data(&self) -> HashMap<String, String> {
        let data = self.data.iter().filter_map(|(k, v)| {
            let v = match v {
                Data::Simple(value) => Some(value),
                _ => None,
            };
            v.map(|v| (k.to_owned(), v.to_owned()))
        });

        if self
            .core
            .as_ref()
            .map(|c| c.auto_capitalize)
            .unwrap_or(true)
        {
            data.clone()
                .chain(data.clone().filter_map(|(k, v)| {
                    k.chars()
                        .next()?
                        .is_lowercase()
                        .then(|| (k[0..1].to_uppercase() + &k[1..], v.to_uppercase()))
                }))
                // We overwrite the auto capitalization
                .chain(data.filter_map(|(k, v)| k.chars().next()?.is_uppercase().then_some((k, v))))
                .collect()
        } else {
            data.collect()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_file() {
        use crate::config::Config;
        use std::path::Path;

        let conf = Config::from_file(Path::new("./data/config_sample.toml")).unwrap();
        assert_eq!(conf.core.as_ref().unwrap().buffer_size, 12);
        assert!(!conf.core.as_ref().unwrap().auto_capitalize);

        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 19);

        // parsing error
        let conf = Config::from_file(Path::new("./data/invalid.toml"));
        assert!(conf.is_err());

        // config file not found
        let conf = Config::from_file(Path::new("./data/not_found"));
        assert!(conf.is_err());

        // data and and core not provided
        let conf = Config::from_file(Path::new("./data/blank_sample.toml")).unwrap();
        let data = conf.extract_data();
        assert_eq!(data.keys().len(), 0);
    }
}
