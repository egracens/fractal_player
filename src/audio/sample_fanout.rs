use std::time::Duration;

use rodio::Source;
use rodio::cpal::Sample;

use crate::audio::{FFTProcessor, PlaybackTracker};

pub struct SampleFanout<S>
where
    S: Source,
    S::Item: Sample,
{
    inner: S,
    tracker: PlaybackTracker,
    fft_proc: FFTProcessor,
    is_playing: bool,
}

impl<S> SampleFanout<S>
where
    S: Source,
    S::Item: Sample,
{
    pub fn new(inner: S, tracker: PlaybackTracker, fft_proc: FFTProcessor) -> Self {
        Self {
            inner,
            tracker,
            fft_proc,
            is_playing: true,
        }
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
        self.tracker.send_snapshot(playing);
    }
}

impl<S> Iterator for SampleFanout<S>
where
    S: Source,
    S::Item: Sample,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;
        let as_f32 = sample.to_float_sample();
        self.tracker.on_frame(self.is_playing);
        self.fft_proc.on_sample(as_f32);
        Some(sample)
    }
}

impl<S> Source for SampleFanout<S>
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
