use bevy::prelude::*;
use bevy_egui::egui;
use serde::Deserialize;

use crate::engine::design_scale::DesignScale;

pub const ASCII_FONT_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(71, 71, 71, 64);
/// Base font size used to measure and scale the ASCII background animation.
pub const ASCII_BACKGROUND_FONT_SIZE: f32 = 12.0;

#[derive(Resource, Debug, Clone)]
pub struct RipperAsciiAnimationHandle(pub Handle<AsciiAnimation>);

#[derive(Resource, Debug, Clone)]
pub struct DecrypterAsciiAnimationHandle(pub Handle<AsciiAnimation>);

#[derive(Deserialize, Asset, Reflect, Debug, Clone)]
pub struct AsciiAnimation {
    animation: AsciiAnimationConfig,
    frames: Vec<AsciiFrame>,
}

impl AsciiAnimation {
    pub fn background_ascii_animation(&self, elapsed: f32) -> String {
        let frame_rate = self.animation.frame_rate.max(1) as f32;
        let frame_count = self.frames.len();
        if frame_count == 0 {
            return String::new();
        }
        let total_duration = frame_count as f32 / frame_rate;
        let t = if self.animation.looping {
            elapsed % total_duration
        } else {
            elapsed.min(total_duration)
        };
        let frame_index = (t * frame_rate) as usize % frame_count;
        self.frames[frame_index].content_string.clone()
    }
}

#[derive(Deserialize, Reflect, Debug, Clone)]
pub struct AsciiAnimationConfig {
    #[serde(rename = "frameRate")]
    frame_rate: u32,
    #[serde(default = "default_looping")]
    looping: bool,
}

fn default_looping() -> bool {
    true
}

#[derive(Deserialize, Reflect, Debug, Clone)]
pub struct AsciiFrame {
    #[serde(rename = "contentString")]
    content_string: String,
}

/// Draws a full-rect ASCII animation behind the window content.
pub fn render_ascii_animation(
    ui: &mut egui::Ui,
    scale: &DesignScale,
    animation: &AsciiAnimation,
    elapsed: f32,
) {
    let rect = ui.max_rect();
    let frame = animation.background_ascii_animation(elapsed);

    // Scale the frame so it covers the whole content area, cropping excess.
    let reference_font = egui::FontId::monospace(scale.py(ASCII_BACKGROUND_FONT_SIZE));
    let reference_galley =
        ui.painter()
            .layout_no_wrap(frame.clone(), reference_font, ASCII_FONT_COLOR);
    let ref_size = reference_galley.rect.size();
    let scale_factor = (rect.width() / ref_size.x).max(rect.height() / ref_size.y);
    let scaled_font = egui::FontId::monospace(scale.py(ASCII_BACKGROUND_FONT_SIZE) * scale_factor);
    let scaled_galley = ui
        .painter()
        .layout_no_wrap(frame, scaled_font, ASCII_FONT_COLOR);
    let pos = rect.center() - scaled_galley.rect.size() * 0.5;

    // Clip to the window content area so the scaled galley cannot spill outside.
    let clipped_painter = ui.painter_at(rect);
    clipped_painter.galley(pos, scaled_galley, ASCII_FONT_COLOR);
}
