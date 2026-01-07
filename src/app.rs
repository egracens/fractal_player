use log::warn;

use crate::{audio::AudioRuntime, state::AppState, ui};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FractalPlayer {
    state: AppState,
    #[serde(skip)]
    audio: AudioRuntime,
}

impl Default for FractalPlayer {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            audio: AudioRuntime::new(),
        }
    }
}

impl FractalPlayer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state: AppState = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            AppState::default()
        };

        Self {
            state,
            audio: AudioRuntime::new(),
        }
    }

    fn handle_open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            let path_str = path.display().to_string();
            self.state.set_audio_file(path_str.clone());
            self.audio.load_file(path_str);
        }
    }

    fn handle_play(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }

        self.state.request_play();
        self.audio.play();

        warn!("Play!");
    }

    fn handle_pause(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }

        self.state.request_pause();
        self.audio.pause();
    }

    fn handle_stop(&mut self) {
        if !self.state.has_audio_file() {
            return;
        }

        self.state.request_stop();
        self.audio.stop();
    }
}

impl eframe::App for FractalPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut actions = ui::UiActions::default();

        ui::top_bar(ctx, &self.state, &mut actions);
        ui::central_panel(ctx, &self.state, &mut actions);

        if actions.open_file {
            self.handle_open_file();
            ctx.request_repaint();
        }

        if actions.play {
            self.handle_play();
            ctx.request_repaint();
        }

        if actions.pause {
            self.handle_pause();
            ctx.request_repaint();
        }

        if actions.stop {
            self.handle_stop();
            ctx.request_repaint();
        }
    }
}
