use flume::Sender;

use crate::audio::{DefaultAnalyzer, SampleConsumer, SpectrogramBins, SpectrogramSlice};

pub struct FFTProcessor {
    analyzer: DefaultAnalyzer,
    buffer: Vec<f32>,
    frame_accum: Vec<f32>,
    channels: u16,
    tx: Sender<SpectrogramBins>,
    sample_rate_hz: u32,
}

impl FFTProcessor {
    pub fn new(channels: u16, analyzer: DefaultAnalyzer, tx: Sender<SpectrogramBins>, sample_rate_hz: u32) -> Self {
        let window_size = analyzer.window_size();

        Self {
            analyzer,
            buffer: Vec::with_capacity(window_size),
            frame_accum: Vec::with_capacity(channels as usize),
            channels: channels.max(1),
            tx,
            sample_rate_hz,
        }
    }
}

impl SampleConsumer for FFTProcessor {
    fn on_sample(&mut self, sample: f32) {
        self.frame_accum.push(sample);

        if self.frame_accum.len() as u16 == self.channels {
            let mono = self.frame_accum.iter().sum::<f32>() / (self.channels as f32);

            self.frame_accum.clear();
            self.buffer.push(mono);

            if self.buffer.len() >= self.analyzer.window_size() {
                let bins = self.analyzer.analyze(&self.buffer[..self.analyzer.window_size()]);
                let slice = SpectrogramSlice {
                    bins,
                    sample_rate_hz: self.sample_rate_hz,
                    window_size: self.analyzer.window_size(),
                };
                let _ = self.tx.send(slice);

                self.buffer.clear();
            }
        }
    }
}
