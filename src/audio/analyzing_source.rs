use std::time::Duration;

use flume::Sender;
use rodio::{cpal::Sample, Source};

use crate::audio::{AnalyzerBins, SpectrogramBins};

pub struct AnalyzingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    inner: S,
    fft: AnalyzerBins,
    spectrogram_tx: Sender<SpectrogramBins>,
    buffer: Vec<f32>,
    frame_accum: Vec<f32>,
    channels: u16,
    window_size: usize,
}

impl<S> AnalyzingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    pub fn new(inner: S, fft: FFTAnalyzer, spectrogram_tx: Sender<SpectrogramBins>) -> Self {
        let window_size = fft.window_size();
        let channels = inner.channels().max(1);
        Self {
            inner,
            fft,
            spectrogram_tx,
            buffer: Vec::with_capacity(window_size),
            frame_accum: Vec::with_capacity(channels as usize),
            channels,
            window_size,
        }
    }
}

impl<S> Iterator for AnalyzingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        let sample_f32 = sample.to_float_sample();

        // Build mono sample by averaging one frame (all channels).
        self.frame_accum.push(sample_f32);
        if self.frame_accum.len() as u16 == self.channels {
            let mono = self.frame_accum.iter().sum::<f32>() / (self.channels as f32);
            self.frame_accum.clear();

            self.buffer.push(mono);
            if self.buffer.len() >= self.window_size {
                let slice = self.fft.analyze(&self.buffer[..self.window_size]);
                let _ = self.spectrogram_tx.send(slice);
                self.buffer.clear();
            }

            Some(mono)
        } else {
            // Continue until a full frame is assembled.
            Some(sample_f32)
        }
    }
}

impl<S> Source for AnalyzingSource<S>
where
    S: Source,
    S::Item: Sample,
{
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
