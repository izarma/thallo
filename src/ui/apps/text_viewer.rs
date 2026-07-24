use bevy_egui::egui::{self, FontFamily, FontId, Frame, Margin, ScrollArea, TextBuffer};

use crate::{
    engine::design_scale::DesignScale,
    ui::theme::palette::{CONTENT_FONT_SIZE, LABEL_COLOR},
};

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
                    let data_id = ui.id().with("text_viewer_selection");

                    let mut text = content;
                    let output = egui::TextEdit::multiline(&mut text)
                        .font(FontId::new(
                            CONTENT_FONT_SIZE * scale.x,
                            FontFamily::Proportional,
                        ))
                        .text_color(LABEL_COLOR)
                        .frame(false)
                        .margin(egui::Margin::symmetric(0, 0))
                        .desired_width(f32::INFINITY)
                        .show(ui);

                    if let Some(cursor_range) = output.cursor_range {
                        let selected = text.char_range(cursor_range.as_sorted_char_range());
                        if !selected.is_empty() {
                            ui.ctx()
                                .data_mut(|d| d.insert_temp(data_id, selected.to_string()));
                        }
                    }

                    output.response.context_menu(|ui| {
                        let selected = ui
                            .ctx()
                            .data(|d| d.get_temp::<String>(data_id))
                            .unwrap_or_default();
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(selected);
                            ui.close();
                        }
                    });
                });
        });
}
