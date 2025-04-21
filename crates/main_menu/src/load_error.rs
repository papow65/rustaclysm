use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LoadError {
    message: String,
}

impl LoadError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
