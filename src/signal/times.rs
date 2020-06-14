#[derive(Debug, Clone)]
pub struct Times {
    step: usize,
    rate: f32,
}

impl Times {
    pub fn new(rate: f32) -> Self {
        Times { step: 0, rate }
    }
}

impl Iterator for Times {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let now = self.step;
        self.step += 1;
        Some((now as f32) / self.rate)
    }
}
