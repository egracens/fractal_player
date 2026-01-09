use crate::{app_state::AppState, audio::AudioRuntime};

pub struct Controller<'a> {
    pub state: &'a mut AppState,
    pub audio: &'a mut AudioRuntime,
}

impl<'a> Controller<'a> {
    pub fn handle_ui_events(
        &mut self,
        events: impl IntoIterator<Item = crate::ui::UiEvent>,
        ctx: &egui::Context,
    ) {
        for ev in events {
            match ev {
                crate::ui::UiEvent::OpenFile(path) => self.load_file(path),
                crate::ui::UiEvent::Play => self.play(),
                crate::ui::UiEvent::Pause => self.pause(),
                crate::ui::UiEvent::Stop => self.stop(),
            }
        }
        ctx.request_repaint();
    }

    pub fn load_file(&mut self, path: String) {
        self.state.set_audio_file(path.clone());
        self.audio.load_file(path);
    }

    pub fn play(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }
        self.state.request_play();
        self.audio.play();
    }

    pub fn pause(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }
        self.state.request_pause();
        self.audio.pause();
    }

    pub fn stop(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }
        self.state.request_stop();
        self.audio.stop();
    }

    pub fn poll_playback(&mut self) {
        self.poll_playback_progress();
        self.poll_spectrogram();
    }

    fn poll_playback_progress(&mut self) {
        while let Ok(snap) = self.audio.progress().try_recv() {
            self.state.is_playing = snap.is_playing;
            self.state.playback_pos_secs = snap.pos_secs as f32;
            self.state.playback_duration_secs = snap.duration_secs as f32;
        }
    }

    fn poll_spectrogram(&mut self) {
        while let Ok(slice) = self.audio.spectrogram().try_recv() {
            self.state.push_spectrogram_slice(slice);
        }
    }
}
