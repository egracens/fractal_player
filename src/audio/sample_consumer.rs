use std::time::Duration;

use rodio::Source;
use rodio::cpal::Sample;

pub trait SampleProducer: Send {
    type Sample: Sample;

    fn next_sample(&mut self) -> Option<Self::Sample>;
    fn channels(&self) -> u16;
    fn sample_rate(&self) -> u32;
    fn total_duration(&self) -> Option<Duration>;
}

impl<T> SampleProducer for T
where
    T: Source + Send,
    T::Item: Sample,
{
    type Sample = T::Item;

    fn next_sample(&mut self) -> Option<Self::Sample> {
        Iterator::next(self)
    }

    fn channels(&self) -> u16 {
        Source::channels(self)
    }

    fn sample_rate(&self) -> u32 {
        Source::sample_rate(self)
    }

    fn total_duration(&self) -> Option<Duration> {
        Source::total_duration(self)
    }
}

pub trait SampleConsumer: Send {
    fn on_sample(&mut self, sample: f32);
    fn on_state_change(&mut self, _is_playing: bool) {}
}
