use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct XtaskError {
    msg: String,
}

impl XtaskError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}

impl Display for XtaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.msg)
    }
}

impl Error for XtaskError {}
