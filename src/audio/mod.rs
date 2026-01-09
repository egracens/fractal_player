mod audio_manager;
mod audio_worker;
mod fft_analyzer;
mod fft_processor;
mod playback_tracker;
mod sample_consumer;
mod sample_fanout;
pub use audio_manager::AudioManager;
pub use fft_analyzer::{DefaultAnalyzer, SpectrogramBins, SpectrogramSlice};
pub use fft_processor::FFTProcessor;
pub use playback_tracker::PlaybackTracker;
pub use sample_consumer::{SampleConsumer, SampleProducer};
pub use sample_fanout::SampleFanout;
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
