use eframe::Storage;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct AppState {
    pub last_file: Option<String>,
    pub is_playing: bool,

    pub playback_pos_secs: f32,
    pub playback_duration_secs: f32,
}

impl AppState {
    pub fn restore(storage: Option<&dyn Storage>) -> AppState {
        if let Some(storage) = storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::default()
        }
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
