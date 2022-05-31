use crate::error;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;

#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub libraries: HashMap<String, Libraries>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Libraries {
    pub command: String,
    pub filter: Filter,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Filter {
    pub directories: Vec<String>,
    pub mime_type_regexes: Option<Vec<String>>,
}

impl Config {
    #[allow(dead_code)]
    pub fn new(config_path: &str) -> std::result::Result<Config, error::Error> {
        let contents = match fs::read_to_string(config_path) {
            Ok(c) => c,
            Err(e) => {
                return Err(error::Error::new(format!(
                    "Unable to read config '{}' contents: {}",
                    config_path, e
                )))
            }
        };

        let result = toml::from_str(&contents);
        match result {
            Ok(c) => Ok(c),
            Err(e) => Err(error::Error::new(format!(
                "Couldn't parse config '{}' as TOML: {}",
                config_path, e
            ))),
        }
    }
}
