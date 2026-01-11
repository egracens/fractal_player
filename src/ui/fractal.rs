use egui::{Context, Ui};

use crate::app_state::AppState;

use super::{UiActions, View};

pub struct Fractal;

impl Fractal {
    pub fn new() -> Self {
        Self
    }

    fn draw_placeholder(&self, ui: &mut Ui, _state: &AppState) {
        ui.label("Fractal visualization will appear here");
        ui.separator();
        ui.label("Waiting for fractal data…");
    }
}

impl View for Fractal {
    fn ui(&self, ctx: &Context, state: &AppState, _actions: &mut UiActions) {
        egui::Window::new("Fractal")
            .default_size([480.0, 260.0])
            .show(ctx, |ui| {
                self.draw_placeholder(ui, state);
            });
    }
}
