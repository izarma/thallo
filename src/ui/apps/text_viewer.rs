use bevy_egui::egui::{self, ScrollArea};

pub(crate) fn show_text_viewer(ui: &mut egui::Ui, content: &str) {
    // a background art maybe nice?
    ScrollArea::vertical()
        .max_height(f32::INFINITY)
        .show(ui, |ui| {
            ui.add_space(30.0);
            ui.label(content);
            ui.separator();
        });
}
