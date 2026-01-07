#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct AppState {
    pub last_file: Option<String>,
    pub is_playing: bool,
}

impl AppState {
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
