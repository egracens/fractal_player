use std::time::Duration;

use flume::Receiver;
use rodio::Source;
use rodio::cpal::Sample;

use crate::audio::{SampleConsumer, SampleProducer};

pub struct SampleFanout<P>
where
    P: SampleProducer,
    P::Sample: Sample<Float = f32>,
{
    inner: P,
    consumers: Vec<Box<dyn SampleConsumer>>,
    is_playing: bool,
    state_rx: Option<Receiver<bool>>,
}

impl<P> SampleFanout<P>
where
    P: SampleProducer,
    P::Sample: Sample<Float = f32>,
{
    #[allow(dead_code)]
    pub fn new(inner: P, consumers: Vec<Box<dyn SampleConsumer>>) -> Self {
        Self {
            inner,
            consumers,
            is_playing: true,
            state_rx: None,
        }
    }

    pub fn with_state_channel(
        inner: P,
        consumers: Vec<Box<dyn SampleConsumer>>,
        state_rx: Receiver<bool>,
    ) -> Self {
        Self {
            inner,
            consumers,
            is_playing: true,
            state_rx: Some(state_rx),
        }
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
        for consumer in self.consumers.iter_mut() {
            consumer.on_state_change(playing);
        }
    }
}

impl<P> Iterator for SampleFanout<P>
where
    P: SampleProducer,
    P::Sample: Sample<Float = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rx) = self.state_rx.as_mut() {
            let mut pending_state: Option<bool> = None;
            while let Ok(state) = rx.try_recv() {
                pending_state = Some(state);
            }
            if let Some(state) = pending_state {
                self.set_playing(state);
            }
        }
        let sample = self.inner.next_sample()?;
        let as_f32 = sample.to_float_sample();
        for consumer in self.consumers.iter_mut() {
            consumer.on_sample(as_f32);
        }
        Some(as_f32)
    }
}

impl<P> Source for SampleFanout<P>
where
    P: SampleProducer,
    P::Sample: Sample<Float = f32>,
{
    fn current_span_len(&self) -> Option<usize> {
        None
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
