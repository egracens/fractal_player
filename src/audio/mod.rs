mod audio_runtime;
mod fft_analyzer;
mod fft_processor;
mod playback_tracker;
mod sample_fanout;
mod sample_consumer;
mod playback_loop;
pub use audio_runtime::AudioRuntime;
pub use fft_analyzer::{AnalyzerBins, SpectrogramBins, SpectrogramSlice, FFT_SIZE};
pub use fft_processor::FFTProcessor;
pub use playback_tracker::PlaybackTracker;
pub use sample_fanout::SampleFanout;
pub use sample_consumer::{SampleConsumer, SampleProducer};
#[derive(Debug)]
pub enum AudioCommand {
    LoadFile(String),
    Play,
    Pause,
    Stop,
    Terminate,
}

#[derive(Clone, Debug, Default)]
pub struct PlaybackSnapshot {
    pub pos_secs: f64,
    pub duration_secs: f64,
    pub is_playing: bool,
}
