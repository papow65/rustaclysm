pub struct StdInstant(std::time::Instant);

impl Default for StdInstant {
    fn default() -> Self {
        Self(std::time::Instant::now())
    }
}

impl StdInstant {
    pub fn next(&mut self) -> std::time::Duration {
        let previous = self.0;
        self.0 = std::time::Instant::now();
        self.0 - previous
    }
}
