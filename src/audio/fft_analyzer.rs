use std::sync::Arc;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

pub const SPECTROGRAM_BINS: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct SpectrogramSlice<const N: usize = SPECTROGRAM_BINS> {
    pub bins: [f32; N],
    pub sample_rate_hz: u32,
    pub window_size: usize,
}

pub type SpectrogramBins = SpectrogramSlice<SPECTROGRAM_BINS>;
pub type DefaultAnalyzer = FFTAnalyzer<SPECTROGRAM_BINS>;

impl<const N: usize> Default for SpectrogramSlice<N> {
    fn default() -> Self {
        Self {
            bins: [0.0; N],
            sample_rate_hz: 44100, // Default sample rate
            window_size: 1024,     // Default window size
        }
    }
}

pub struct FFTAnalyzer<const N: usize = SPECTROGRAM_BINS> {
    window_size: usize,
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
}

impl<const N: usize> FFTAnalyzer<N> {
    pub fn new(window_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(window_size);
        let scratch = vec![Complex::ZERO; fft.get_inplace_scratch_len()];
        let buffer = vec![Complex::ZERO; window_size];
        let window = hann_window(window_size);

        Self {
            window_size,
            fft,
            scratch,
            buffer,
            window,
        }
    }

    pub fn analyze(&mut self, samples: &[f32]) -> [f32; N] {
        // Copy & window into buffer, zero-pad if short.
        for (i, slot) in self.buffer.iter_mut().enumerate() {
            let s = samples.get(i).copied().unwrap_or(0.0);
            *slot = Complex::new(s * self.window[i], 0.0);
        }

        // Run FFT in-place.
        self.fft
            .process_with_scratch(&mut self.buffer, &mut self.scratch);

        // Magnitudes (first half) and bin downsample.
        let half = self.window_size / 2;
        let mut bins = [0.0f32; N];
        let mut acc = vec![0.0f32; N];
        let mut counts = vec![0usize; N];

        for (i, c) in self.buffer.iter().take(half).enumerate() {
            let mag = c.norm();
            let band = i * N / half;
            if band < N {
                acc[band] += mag;
                counts[band] += 1;
            }
        }

        for (i, out) in bins.iter_mut().enumerate() {
            let sum = acc.get(i).copied().unwrap_or(0.0);
            let n = counts.get(i).copied().unwrap_or(1).max(1) as f32;
            *out = sum / n;
        }

        bins
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }
}

fn hann_window(n: usize) -> Vec<f32> {
    let n_f = n as f32;
    (0..n)
        .map(|i| {
            let x = i as f32;
            0.5 * (1.0 - (2.0 * std::f32::consts::PI * x / (n_f - 1.0)).cos())
        })
        .collect()
}
