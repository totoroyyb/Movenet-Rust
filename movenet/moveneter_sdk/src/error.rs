use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RecogError {
    info: String
}

impl RecogError {
    pub fn new(msg: &str) -> RecogError {
        RecogError{ info: msg.to_string() }
    }
}

impl fmt::Display for RecogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Error for RecogError {
    fn description(&self) -> &str {
        &self.info
    }
}
