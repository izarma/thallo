use bevy_egui::egui::{self, Color32};

// Colors
pub const HEADER_COLOR: Color32 = Color32::from_rgb(240, 240, 240);
pub const LABEL_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
pub const LABEL_META: Color32 = Color32::from_rgb(110, 90, 90);
pub const BUTTON_BG: Color32 = Color32::from_rgb(50, 50, 60);
pub const BUTTON_HOVERED_BG: Color32 = Color32::from_rgb(230, 70, 85);
pub const BUTTON_ACTIVE_BG: Color32 = Color32::from_rgb(90, 90, 110);
pub const COLOR_DONE: Color32 = Color32::from_rgb(80, 200, 120);
pub const COLOR_DONE_FILL: Color32 = Color32::from_rgb(60, 200, 100);
pub const COLOR_PROGRESS_BORDER: Color32 = Color32::from_rgb(100, 100, 120);
pub const CHATBOX_ANON: Color32 = Color32::from_rgb(200, 80, 80);
pub const DEEP_RED_THEME: Color32 = Color32::from_rgb(30, 5, 5);
pub const RED_CONTRAST_THEME: Color32 = Color32::from_rgb(210, 160, 160);
pub const DECRYPT_THEME: Color32 = Color32::from_rgb(80, 200, 110);
pub const RIPPER_THEME: Color32 = Color32::from_rgb(80, 160, 200);

// Font sizes
pub const HEADING_FONT_SIZE: f32 = 40.0;
pub const CONTENT_FONT_SIZE: f32 = 24.0;
pub const SYSTEM_FONT_SIZE: f32 = 16.0;
pub const FONT_CHAT_SM: f32 = 20.0;
pub const FONT_CHAT: f32 = 18.0;
pub const FONT_SMALL: f32 = 12.0;

pub fn apply_button_theme(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    v.widgets.inactive.weak_bg_fill = BUTTON_BG;
    v.widgets.hovered.weak_bg_fill = BUTTON_HOVERED_BG;
    v.widgets.active.weak_bg_fill = BUTTON_ACTIVE_BG;
    v.selection.bg_fill = DEEP_RED_THEME;
}
