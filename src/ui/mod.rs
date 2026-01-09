pub mod helpers;
pub mod spectrogram;
pub mod player_controls;
pub mod top_bar;

pub use player_controls::PlayerControls;
pub use spectrogram::Spectrogram;
pub use top_bar::TopBar;

use egui::Context;

use crate::app_state::AppState;

#[derive(Default)]
pub struct UiActions {
    pub events: Vec<UiEvent>,
}

pub enum UiEvent {
    OpenFile(String),
    Play,
    Pause,
    Stop,
}

pub trait View {
    fn ui(&self, ctx: &Context, state: &AppState, actions: &mut UiActions);
}
