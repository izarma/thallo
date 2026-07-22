use bevy_egui::egui::{self, Frame, Margin, ScrollArea};

use crate::{engine::design_scale::DesignScale, ui::theme::widgets::primitives};

pub(crate) fn show_text_viewer(ui: &mut egui::Ui, content: &str, scale: &DesignScale) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::default()
                .inner_margin(Margin {
                    left: (20.0 * scale.x) as i8,
                    right: (16.0 * scale.x) as i8,
                    top: (20.0 * scale.y) as i8,
                    bottom: (20.0 * scale.y) as i8,
                })
                .show(ui, |ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    primitives::label(ui, content, scale)
                });
        });
}
