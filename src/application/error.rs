use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LoadError {
    lines: Vec<String>,
}

impl LoadError {
    pub(crate) fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}
