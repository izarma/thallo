use bevy_egui::egui;

use crate::engine::{design_scale::DesignScale, file_system::FsPath};

const SEGMENT_FONT_DESIGN: f32 = 16.0;
const ADDR_LEFT_PAD_DESIGN: f32 = 8.0;
const ITEM_SPACING_DESIGN: f32 = 8.0;

/// An address bar with a back button.
/// Returns `true` if the back button was clicked.
pub fn address_bar(
    ui: &mut egui::Ui,
    path: &FsPath,
    can_go_back: bool,
    scale: &DesignScale,
) -> bool {
    let mut go_back = false;
    let font_size = scale.py(SEGMENT_FONT_DESIGN);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = scale.uniform() * ITEM_SPACING_DESIGN;
        ui.add_space(scale.uniform() * ADDR_LEFT_PAD_DESIGN);
        if ui
            .add_enabled(
                can_go_back,
                egui::Button::new(egui::RichText::new("◀").size(font_size)),
            )
            .clicked()
        {
            go_back = true;
        }
        ui.separator();
        let segments: Vec<&str> = path.segments().collect();
        for (i, segment) in segments.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new("/").weak().size(font_size));
            }
            if i == segments.len() - 1 {
                ui.label(egui::RichText::new(*segment).strong().size(font_size));
            } else {
                ui.label(egui::RichText::new(*segment).weak().size(font_size));
            }
        }
    });
    go_back
}
