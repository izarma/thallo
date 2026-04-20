use bevy_egui::egui::{self, Color32};

pub const HEADER_COLOR: Color32 = Color32::from_rgb(240, 240, 240);
pub const LABEL_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
pub const BUTTON_TEXT_COLOR: Color32 = Color32::from_rgb(240, 240, 240);
pub const BUTTON_BG: Color32 = Color32::from_rgb(50, 50, 60);
pub const BUTTON_HOVERED_BG: Color32 = Color32::from_rgb(230, 70, 85);
pub const BUTTON_ACTIVE_BG: Color32 = Color32::from_rgb(90, 90, 110);

pub fn apply_button_theme(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    v.widgets.inactive.weak_bg_fill = BUTTON_BG;
    v.widgets.hovered.weak_bg_fill = BUTTON_HOVERED_BG;
    v.widgets.active.weak_bg_fill = BUTTON_ACTIVE_BG;
}
