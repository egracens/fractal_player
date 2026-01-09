use crate::{
    app_state::AppState,
    audio::AudioRuntime,
    controller::Controller,
    ui::{UiActions, View, player_controls::PlayerControlsView, top_bar::TopBarView},
};

pub struct FractalPlayer {
    state: AppState,
    audio: AudioRuntime,
    views: Vec<Box<dyn View>>,
}

impl FractalPlayer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state: AppState = AppState::restore(cc.storage);

        let mut audio = AudioRuntime::new(state.last_file.clone());
        audio.spawn_playback_thread();

        let views: Vec<Box<dyn View>> = vec![
            Box::new(TopBarView::new()),
            Box::new(PlayerControlsView::new()),
        ];

        Self {
            state,
            audio,
            views,
        }
    }
}

impl eframe::App for FractalPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        const PROGRESS_FPS: u64 = 144;
        const PROGRESS_FRAME_MS: u64 = 1000 / PROGRESS_FPS;

        let mut actions = UiActions::default();

        for view in self.views.iter() {
            view.ui(ctx, &self.state, &mut actions);
        }

        let mut controller = Controller {
            state: &mut self.state,
            audio: &mut self.audio,
        };

        controller.handle_ui_events(actions.events.drain(..), ctx);

        controller.poll_playback_progress();

        ctx.request_repaint_after(std::time::Duration::from_millis(PROGRESS_FRAME_MS));
    }
}
