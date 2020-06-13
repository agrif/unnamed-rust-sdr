#[derive(Debug, Clone)]
pub struct Times {
    step: usize,
    rate: f64,
}

impl Times {
    pub fn new(rate: f64) -> Self {
        Times { step: 0, rate }
    }
}

impl Iterator for Times {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        let now = self.step;
        self.step += 1;
        Some((now as f64) / self.rate)
    }
}
