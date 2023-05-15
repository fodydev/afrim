use serde::Deserialize;
use std::{collections::HashMap, error, fs, path::Path};
use toml::{self};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub core: Option<CoreConfig>,
    pub data: HashMap<String, Data>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CoreConfig {
    pub buffer_size: usize,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Data {
    Simple(String),
    File(DataFile),
    Detailed(DetailedData),
}

#[derive(Deserialize, Debug, Clone)]
pub struct DataFile {
    path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DetailedData {
    value: String,
    alias: Vec<String>,
}

impl Config {
    pub fn from_file(filepath: &Path) -> Result<Self, Box<dyn error::Error>> {
        let content = fs::read_to_string(filepath)?;
        let mut config: Self = toml::from_str(&content)?;

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

    pub fn extract_data(&self) -> HashMap<&String, &String> {
        self.data
            .iter()
            .filter_map(|(k, v)| {
                let v = match v {
                    Data::Simple(value) => Some(value),
                    _ => None,
                };
                v.map(|v| (k, v))
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

        let conf = Config::from_file(Path::new("./data/config_sample.toml"));

        let conf = conf.unwrap();
        assert_eq!(conf.core.unwrap().buffer_size, 12);
        assert_eq!(conf.data.len(), 19);

        let conf = Config::from_file(Path::new("./not_found"));
        assert!(conf.is_err());

        let conf = Config::from_file(Path::new("./data/blank_sample.toml"));
        assert!(conf.is_err());
    }
}
