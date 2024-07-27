use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An io Error was thrown while reading the config")]
    Io(#[from] io::Error),
    #[error("An Error was thrown while trying to parse the config as TOML")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub libraries: HashMap<String, Libraries>,
    pub fs_watch: Option<FsWatch>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FsWatch {
    pub min_command_exec_freq: Option<u64>,
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
    pub fn new(config_path: &String) -> std::result::Result<Config, Error> {
        let contents = fs::read_to_string(config_path)?;

        Ok(toml::from_str(&contents)?)
    }
}
