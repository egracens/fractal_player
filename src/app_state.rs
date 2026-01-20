use eframe::Storage;

use crate::audio::SpectrogramBins;

const DEFAULT_SPECTROGRAM_CAPACITY: usize = 200;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct AppState {
    pub last_file: Option<String>,
    pub is_playing: bool,

    pub playback_pos_secs: f32,
    pub playback_duration_secs: f32,

    pub fractal_type: FractalType,

    #[serde(skip)]
    pub spectrogram: Vec<SpectrogramBins>,
    #[serde(skip)]
    pub spectrogram_capacity: usize,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FractalType {
    #[default]
    Triangle,
    Aurora,
    Mandelbrot,
}

impl AppState {
    pub fn restore(storage: Option<&dyn Storage>) -> AppState {
        if let Some(storage) = storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::default()
        }
    }

    pub fn init_runtime_fields(&mut self) {
        if self.spectrogram_capacity == 0 {
            self.spectrogram_capacity = DEFAULT_SPECTROGRAM_CAPACITY;
        }
        if self.spectrogram.is_empty() {
            self.spectrogram.reserve(self.spectrogram_capacity);
        }
    }

    pub fn push_spectrogram_slice(&mut self, slice: SpectrogramBins) {
        if self.spectrogram.len() >= self.spectrogram_capacity {
            self.spectrogram.remove(0);
        }
        self.spectrogram.push(slice);
    }

    pub fn has_audio_file(&self) -> bool {
        self.last_file.is_some()
    }

    pub fn set_audio_file(&mut self, path: String) {
        self.last_file = Some(path);
        self.is_playing = false;
    }

    pub fn request_play(&mut self) {
        self.is_playing = true;
    }

    pub fn request_pause(&mut self) {
        self.is_playing = false;
    }

    pub fn request_stop(&mut self) {
        self.is_playing = false;
    }
}
