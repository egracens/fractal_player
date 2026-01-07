use egui::Context;

use crate::state::AppState;

#[derive(Default)]
pub struct UiActions {
    pub open_file: bool,
    pub play: bool,
    pub pause: bool,
    pub stop: bool,
}

pub fn top_bar(ctx: &Context, _state: &AppState, actions: &mut UiActions) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open…").clicked() {
                    actions.open_file = true;
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    });
}

pub fn central_panel(ctx: &Context, state: &AppState, actions: &mut UiActions) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Fractal player");

        ui.horizontal(|ui| {
            if ui.button("Open file…").clicked() {
                actions.open_file = true;
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
                actions.play = true;
            }

            if ui
                .add_enabled(has_file, egui::Button::new("Pause"))
                .clicked()
            {
                actions.pause = true;
            }

            if ui
                .add_enabled(has_file, egui::Button::new("Stop"))
                .clicked()
            {
                actions.stop = true;
            }

            if !has_file {
                ui.label("Load an audio file to enable playback.");
            }
        });

        ui.separator();

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            powered_by_egui_and_eframe(ui);
            egui::warn_if_debug_build(ui);
        });
    });
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
