mod audio_runtime;
mod playback_loop;
pub use audio_runtime::AudioRuntime;

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

#[derive(Clone, Copy, Debug, Default)]
pub struct SpectrogramSlice {
    pub bins: [f32; 32],
}

impl SpectrogramSlice {
    pub fn new(bins: [f32; 32]) -> Self {
        Self { bins }
    }
}
