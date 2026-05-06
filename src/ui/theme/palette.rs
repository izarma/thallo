use bevy_egui::egui::{self, Color32};

// Core
pub const HEADER_COLOR: Color32 = Color32::from_rgb(240, 240, 240);
pub const LABEL_COLOR: Color32 = Color32::from_rgb(200, 200, 200);
pub const BUTTON_TEXT_COLOR: Color32 = Color32::from_rgb(240, 240, 240);
pub const BUTTON_BG: Color32 = Color32::from_rgb(50, 50, 60);
pub const BUTTON_HOVERED_BG: Color32 = Color32::from_rgb(230, 70, 85);
pub const BUTTON_ACTIVE_BG: Color32 = Color32::from_rgb(90, 90, 110);

// Progress / status
pub const COLOR_DONE: Color32 = Color32::from_rgb(80, 200, 120);
pub const COLOR_DONE_FILL: Color32 = Color32::from_rgb(60, 200, 100);
pub const COLOR_PENDING: Color32 = Color32::from_rgb(55, 55, 70);
pub const COLOR_PROGRESS_BORDER: Color32 = Color32::from_rgb(100, 100, 120);
pub const COLOR_PROGRESS_TRACK: Color32 = Color32::from_rgb(14, 14, 22);

// Terminal
pub const TERMINAL_DIM: Color32 = Color32::from_rgb(120, 120, 120);
pub const TERMINAL_OUTPUT: Color32 = Color32::from_rgb(180, 180, 180);
pub const TERMINAL_PROMPT: Color32 = Color32::from_rgb(140, 160, 140);
pub const TERMINAL_INPUT: Color32 = Color32::from_rgb(210, 210, 210);
pub const TERMINAL_HEADER: Color32 = Color32::from_rgb(100, 110, 100);

// Chatbox
pub const CHATBOX_ANON: Color32 = Color32::from_rgb(200, 80, 80);
pub const CHATBOX_PLAYER: Color32 = Color32::from_rgb(180, 180, 180);
pub const CHATBOX_META: Color32 = Color32::from_rgb(110, 90, 90);
pub const CHATBOX_SEND_TEXT: Color32 = Color32::from_rgb(210, 160, 160);
pub const DEEP_RED_THEME: Color32 = Color32::from_rgb(30, 5, 5);

// Decrypter
pub const DECRYPT_SCAN: Color32 = Color32::from_rgb(80, 200, 110);

// NetRipper
pub const RIPPER_RELAY: Color32 = Color32::from_rgb(80, 160, 200);
pub const RIPPER_RELAY_DONE: Color32 = Color32::from_rgb(60, 200, 120);

// Font sizes
// Full-screen menus
pub const HEADING_FONT_SIZE: f32 = 40.0;
pub const FONT_MENU_LABEL: f32 = 24.0;
pub const FONT_MENU_BUTTON: f32 = 32.0;

// In-app panels (decrypter, ripper, …)
pub const FONT_PANEL_TITLE: f32 = 17.0;
pub const FONT_BODY: f32 = 14.0;
pub const FONT_CAPTION: f32 = 11.0;
pub const FONT_SCAN: f32 = 10.0;
pub const FONT_INDICATOR: f32 = 9.0;

// Chatbox
pub const FONT_CHAT_SM: f32 = 20.0;
pub const FONT_CHAT: f32 = 18.0;

// Terminal (monospace)
pub const FONT_MONO: f32 = 12.0;
pub const FONT_MONO_SM: f32 = 10.5;

// Task-bar (use as a ratio: FONT_* × (tab_h / TASKBAR_DESIGN_H))
pub const FONT_TAB: f32 = 15.0;
pub const FONT_TAB_POPUP: f32 = 14.0;
pub const FONT_MENU_ITEM: f32 = 16.0;
pub const FONT_CLOCK: f32 = 13.0;

// Address bar / misc small labels
pub const FONT_ADDR: f32 = 16.0;
pub const FONT_SMALL: f32 = 10.0;

// Minigame timer
pub const FONT_TIMER: f32 = 20.0;

pub fn apply_button_theme(ui: &mut egui::Ui) {
    let v = ui.visuals_mut();
    v.widgets.inactive.weak_bg_fill = BUTTON_BG;
    v.widgets.hovered.weak_bg_fill = BUTTON_HOVERED_BG;
    v.widgets.active.weak_bg_fill = BUTTON_ACTIVE_BG;
}
