use crate::config;
use crate::mime_type;
use crate::template;
use std::collections;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(test)]
mod tests;

const TEMPLATE_VAR_FILE_PATH: &str = "file_path";
const TEMPLATE_VAR_MIME_TYPE: &str = "mime_type";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A config error was thrown")]
    Config(#[from] config::Error),
    #[error("A MIME type error was thrown")]
    Mime(#[from] mime_type::Error),
    #[error("A templating error was thrown")]
    Template(#[from] template::Error),
    #[error("An IO error was thrown")]
    Io(#[from] std::io::Error),
    #[error("A regex error was thrown")]
    Regex(#[from] regex::Error),
    #[error("Could not read path {0}")]
    ReadPath(Box<Path>),
}

#[derive(Debug)]
pub struct Library<'a> {
    config: &'a config::Libraries,
    skip_running_commands: &'a bool,
}

impl<'a> Library<'a> {
    #[allow(dead_code)]
    pub fn new(config: &'a config::Libraries, skip_running_commands: &'a bool) -> Library<'a> {
        Library {
            config,
            skip_running_commands,
        }
    }

    #[allow(dead_code)]
    pub fn process(&self, path: Option<&Path>) -> Result<u64, Error> {
        let mut num_processed_files: u64 = 0;

        if let Some(p) = path {
            if p.is_dir() {
                return self.process_dir(p);
            } else {
                let mut num_processed = 0;
                if self.process_file(p)? {
                    num_processed += 1;
                }

                return Ok(num_processed);
            }
        }

        for cur_dir in self.config.filter.directories.iter() {
            num_processed_files += self.process_dir(Path::new(cur_dir))?;
        }
        Ok(num_processed_files)
    }

    fn process_dir(&self, dir: &Path) -> Result<u64, Error> {
        let mut num_processed_files: u64 = 0;
        // iteratively go through all files in directory
        let paths = fs::read_dir(dir)?;

        for cur_entry_res in paths {
            let cur_entry = cur_entry_res?;

            let file_type = cur_entry.file_type()?;
            if file_type.is_dir() {
                num_processed_files += self.process_dir(&cur_entry.path())?;
            } else if self.process_file(&cur_entry.path())? {
                num_processed_files += 1;
            }
        }
        Ok(num_processed_files)
    }

    fn process_file(&self, path: &Path) -> Result<bool, Error> {
        let mime_type = match mime_type::File::new(path).get_mime_type() {
            Err(e) => {
                eprintln!("{}", e);
                return Ok(false);
            }
            Ok(m) => m,
        };

        // if mime_type filters are defined, check if file fits any of them
        if let Some(regexes) = &self.config.filter.mime_type_regexes {
            let mut is_matched = false;
            for cur_regex in regexes.iter() {
                let re = regex::Regex::new(cur_regex.as_str())?;

                if re.is_match(mime_type.as_str()) {
                    is_matched = true;
                    break;
                }
            }

            if !is_matched {
                return Ok(false);
            }
        }

        // run the command if mime_type passes
        self.run_command(path, mime_type.as_str())
    }

    fn run_command(&self, path: &Path, mime_type: &str) -> Result<bool, Error> {
        if *self.skip_running_commands {
            match path.as_os_str().to_str() {
                None => {
                    return Err(Error::ReadPath(path.into()));
                }
                Some(s) => {
                    println!("{}", s);
                    return Ok(true);
                }
            }
        }

        let path_str = match path.as_os_str().to_str() {
            None => {
                return Err(Error::ReadPath(path.into()));
            }
            Some(s) => s,
        };
        let mut data = collections::HashMap::new();
        data.insert(TEMPLATE_VAR_FILE_PATH, path_str);
        data.insert(TEMPLATE_VAR_MIME_TYPE, mime_type);

        let tmplt = template::Template::new(self.config.command.clone())?;
        let cmd_str = tmplt.render(&data)?;

        let output = if env::consts::OS == "windows" {
            Command::new("cmd").arg("/C").arg(cmd_str).output()
        } else {
            Command::new("sh").arg("-c").arg(cmd_str).output()
        };
        output?;

        Ok(true)
    }

    pub fn contains_path(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        for cur_dir_path in self.config.filter.directories.iter() {
            let c = match fs::canonicalize(Path::new(cur_dir_path)) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let cur_full_dir_path = match c.as_os_str().to_str() {
                Some(c) => c,
                None => continue,
            };

            let p = match fs::canonicalize(path) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let full_path = match p.as_os_str().to_str() {
                Some(p) => p,
                None => continue,
            };

            if full_path.contains(cur_full_dir_path) {
                return true;
            }
        }

        false
    }
}
