use super::trace;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use egui::Slider;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(ui_system);
    }
}

fn ui_system(mut egui_context: ResMut<EguiContext>, mut settings: ResMut<trace::Settings>) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, [-5.0, 5.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.checkbox(&mut settings.show_ray_steps, "Show ray steps");
            ui.checkbox(&mut settings.freeze, "Freeze");
            ui.checkbox(&mut settings.misc_bool, "Misc bool");
            ui.add(Slider::new(&mut settings.misc_float, 1.0..=100.0).text("Misc float"));
        });
}
