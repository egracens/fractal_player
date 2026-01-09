use flume::Sender;

use crate::audio::{AnalyzerBins, SampleConsumer, SpectrogramBins};

pub struct FFTProcessor {
    analyzer: AnalyzerBins,
    buffer: Vec<f32>,
    frame_accum: Vec<f32>,
    channels: u16,
    tx: Sender<SpectrogramBins>,
}

impl FFTProcessor {
    pub fn new(channels: u16, analyzer: AnalyzerBins, tx: Sender<SpectrogramBins>) -> Self {
        let fft_size = analyzer.fft_size();
        Self {
            analyzer,
            buffer: Vec::with_capacity(fft_size),
            frame_accum: Vec::with_capacity(channels as usize),
            channels: channels.max(1),
            tx,
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

            if self.buffer.len() >= self.analyzer.fft_size() {
                let slice = self
                    .analyzer
                    .analyze(&self.buffer[..self.analyzer.fft_size()]);
                let _ = self.tx.send(slice);

                self.buffer.clear();
            }
        }
    }
}
