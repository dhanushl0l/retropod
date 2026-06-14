use std::sync::{Arc, Mutex};

pub struct Visualizer {
    buffer: Vec<f32>,
    fft_size: usize,
    pub bins: Arc<Mutex<Vec<f32>>>,
}

impl Visualizer {
    pub fn new(fft_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(fft_size),
            fft_size,
            bins: Arc::new(Mutex::new(vec![0.0; fft_size / 2])),
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.buffer.push(sample);
        if self.buffer.len() >= self.fft_size {
            self.compute_fft();
            self.buffer.clear();
        }
    }

    fn compute_fft(&self) {
        return;
    }
}
