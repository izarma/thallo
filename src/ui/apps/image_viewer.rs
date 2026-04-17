use bevy_egui::egui::{self, TextureId, Vec2};

use crate::engine::design_scale::DesignScale;

pub(crate) fn show_image_viewer(
    ui: &mut egui::Ui,
    texture_id: TextureId,
    size: Vec2,
    scale: &DesignScale,
) {
    let scaled_size = scale.px(size.x, size.y);
    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
        ui.label(egui::RichText::new(format!("{}×{}px", size.x as u32, size.y as u32)).size(10.0));
        ui.centered_and_justified(|ui| {
            ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                texture_id,
                scaled_size,
            )));
        });
    });
}
