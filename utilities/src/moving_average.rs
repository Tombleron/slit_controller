pub struct MovingAverage {
    values: Vec<f32>,
    max_size: usize,
}

impl MovingAverage {
    pub fn new(max_size: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add(&mut self, value: f32) {
        if self.values.len() >= self.max_size {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    pub fn get_rms(&self) -> f32 {
        if self.values.is_empty() {
            0.0
        } else {
            let sum_of_squares: f32 = self.values.iter().map(|&v| v * v).sum();
            (sum_of_squares / self.values.len() as f32).sqrt()
        }
    }
}
