use crate::{audio::AudioRuntime, controller::Controller, state::AppState, ui};

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
}

impl eframe::App for FractalPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut actions = ui::UiActions::default();

        ui::top_bar(ctx, &self.state, &mut actions);
        ui::central_panel(ctx, &self.state, &mut actions);

        let mut controller = Controller {
            state: &mut self.state,
            audio: &mut self.audio,
        };

        for event in actions.events.drain(..) {
            match event {
                ui::UiEvent::OpenFile(path) => controller.load_file(path),
                ui::UiEvent::Play => controller.play(),
                ui::UiEvent::Pause => controller.pause(),
                ui::UiEvent::Stop => controller.stop(),
            }
            ctx.request_repaint();
        }
    }
}
