use egui::Context;

use crate::app_state::AppState;

use super::{helpers::pick_file_dialog, UiActions, UiEvent, View};

pub struct PlayerControls;

impl PlayerControls {
    pub fn new() -> Self {
        Self
    }
}

impl View for PlayerControls {
    fn ui(&self, ctx: &Context, state: &AppState, actions: &mut UiActions) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Fractal player");

            ui.horizontal(|ui| {
                if ui.button("Open file…").clicked() {
                    if let Some(path) = pick_file_dialog() {
                        actions.events.push(UiEvent::OpenFile(path));
                    }
                }

                match &state.last_file {
                    Some(path) => ui.label(path),
                    None => ui.label("No file selected."),
                };
            });

            ui.horizontal(|ui| {
                let has_file = state.has_audio_file();

                if ui
                    .add_enabled(has_file, egui::Button::new("Play"))
                    .clicked()
                {
                    actions.events.push(UiEvent::Play);
                }

                if ui
                    .add_enabled(has_file, egui::Button::new("Pause"))
                    .clicked()
                {
                    actions.events.push(UiEvent::Pause);
                }

                if ui
                    .add_enabled(has_file, egui::Button::new("Stop"))
                    .clicked()
                {
                    actions.events.push(UiEvent::Stop);
                }

                if !has_file {
                    ui.label("Load an audio file to enable playback.");
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let dur = state.playback_duration_secs.max(0.1);
                let mut pos = state.playback_pos_secs;

                ui.label("Progress:");
                ui.style_mut().spacing.slider_width = 200.0;
                let slider = ui.add(egui::Slider::new(&mut pos, 0.0..=dur).show_value(false));

                if slider.changed() {
                    actions.events.push(UiEvent::Seek(pos));
                }

                ui.label(format!(
                    "{:.1}s / {:.1}s",
                    state.playback_pos_secs, state.playback_duration_secs
                ));
            });
        });
    }
}
