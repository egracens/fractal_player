use log::info;

use crate::{
    audio::{is_mp3_path, AudioRuntime},
    state::AppState,
};

pub struct Controller<'a> {
    pub state: &'a mut AppState,
    pub audio: &'a mut AudioRuntime,
}

impl<'a> Controller<'a> {
    pub fn load_file(&mut self, path: String) {
        if !is_mp3_path(&path) {
            log::warn!("Controller: non-mp3 selected; ignoring: {path}");
            return;
        }
        info!("Controller: load_file {path}");
        self.state.set_audio_file(path.clone());
        self.audio.load_file(path);
    }

    pub fn play(&mut self) {
        if !self.state.has_audio_file() {
            info!("Controller: play requested with no file loaded; ignoring");
            return;
        }
        info!("Controller: play");
        self.state.request_play();
        self.audio.play();
    }

    pub fn pause(&mut self) {
        if !self.state.has_audio_file() {
            info!("Controller: pause requested with no file loaded; ignoring");
            return;
        }
        info!("Controller: pause");
        self.state.request_pause();
        self.audio.pause();
    }

    pub fn stop(&mut self) {
        if !self.state.has_audio_file() {
            info!("Controller: stop requested with no file loaded; ignoring");
            return;
        }
        info!("Controller: stop");
        self.state.request_stop();
        self.audio.stop();
    }
}
