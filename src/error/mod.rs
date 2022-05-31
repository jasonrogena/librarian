use std::error;
use std::fmt;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(msg: String) -> Error {
        Error { message: msg }
    }

    #[allow(dead_code)]
    pub fn get_message(&self) -> &String {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", &self.message)
    }
}

impl error::Error for Error {}
