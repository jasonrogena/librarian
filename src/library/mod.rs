use crate::config;
use crate::error;
use crate::template;
use std::collections;
use std::env;
use std::fs;
use std::process::Command;

mod mime_type;
#[cfg(test)]
mod tests;

const TEMPLATE_VAR_FILE_PATH: &str = "file_path";
const TEMPLATE_VAR_MIME_TYPE: &str = "mime_type";

#[derive(Debug)]
pub struct Library {
    config: config::Libraries,
}

impl Library {
    #[allow(dead_code)]
    pub fn new(config: config::Libraries) -> Library {
        Library { config }
    }

    #[allow(dead_code)]
    pub fn process(&self) -> Result<u64, error::Error> {
        let mut num_processed_files: u64 = 0;
        for cur_dir in self.config.filter.directories.iter() {
            num_processed_files += self.process_dir(cur_dir)?;
        }
        Ok(num_processed_files)
    }

    fn process_dir(&self, dir: &str) -> Result<u64, error::Error> {
        let mut num_processed_files: u64 = 0;
        // iteratively go through all files in directory
        let paths = match fs::read_dir(dir) {
            Err(e) => {
                return Err(error::Error::new(format!(
                    "Unable to read directory contents for '{}': {}",
                    dir, e
                )))
            }
            Ok(p) => p,
        };

        for cur_entry_res in paths {
            let cur_entry = match cur_entry_res {
                Err(e) => {
                    return Err(error::Error::new(format!(
                        "An error occurred reading a directory entry for '{}': {}",
                        dir, e
                    )))
                }
                Ok(de) => de,
            };

            let file_type = match cur_entry.file_type() {
                Err(e) => {
                    return Err(error::Error::new(format!(
                        "Could not determine the file_type of {:?}: {}",
                        cur_entry.path(),
                        e
                    )))
                }
                Ok(f) => f,
            };

            let path = match cur_entry.path().into_os_string().into_string() {
                Ok(s) => s,
                Err(e) => {
                    return Err(error::Error::new(format!(
                        "Could not get the path for {:?} as a string: {:?}",
                        cur_entry, e
                    )))
                }
            };

            if file_type.is_dir() {
                num_processed_files += self.process_dir(&path)?;
            } else if self.process_file(&path)? {
                num_processed_files += 1;
            }
        }
        Ok(num_processed_files)
    }

    fn process_file(&self, path: &str) -> Result<bool, error::Error> {
        let mime_type = match mime_type::File::new(path.to_string()).get_mime_type() {
            Err(e) => {
                return Err(error::Error::new(format!(
                    "Couldn't get MIME type for '{}': {}",
                    path, e
                )))
            }
            Ok(m) => m,
        };

        // if mime_type filters are defined, check if file fits any of them
        if let Some(regexes) = &self.config.filter.mime_type_regexes {
            let mut is_matched = false;
            for cur_regex in regexes.iter() {
                let re = match regex::Regex::new(cur_regex.as_str()) {
                    Err(e) => {
                        return Err(error::Error::new(format!(
                            "Couldn't parse regex '{}' to use to test the MIME type for '{}': {}",
                            cur_regex, path, e
                        )))
                    }
                    Ok(r) => r,
                };

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
        let mut data = collections::HashMap::new();
        data.insert(TEMPLATE_VAR_FILE_PATH, path);
        data.insert(TEMPLATE_VAR_MIME_TYPE, mime_type.as_str());

        let tmplt = template::Template::new(self.config.command.clone())?;
        let cmd_str = tmplt.render(&data)?;

        let output = if env::consts::OS == "windows" {
            Command::new("cmd").arg("/C").arg(cmd_str).output()
        } else {
            Command::new("sh").arg("-c").arg(cmd_str).output()
        };
        match output {
            Err(e) => Err(error::Error::new(format!(
                "Got an error while running command against file '{}': {}",
                path, e
            ))),
            Ok(_) => Ok(true),
        }
    }
}
