use bevy_egui::egui;

use crate::ui::theme::{
    palette::{BUTTON_TEXT_COLOR, apply_button_theme},
    widgets::primitives::empty_state,
};

pub(super) fn show_unlocker(ui: &mut egui::Ui, input: &mut String) -> bool {
    let mut submitted = false;
    ui.vertical_centered(|ui| {
        ui.set_max_width(260.0);
        ui.add_space(16.0);
        empty_state(ui, "This item is Password protected");
        ui.add_space(12.0);
        ui.horizontal_centered(|ui| {
            let field = egui::TextEdit::singleline(input)
                .password(true)
                .hint_text("Enter password")
                .desired_width(180.0);
            ui.add(field);
            apply_button_theme(ui);
            let btn = ui.add_sized(
                [60.0, 18.0],
                egui::Button::new(
                    egui::RichText::new("Unlock")
                        .size(10.0)
                        .color(BUTTON_TEXT_COLOR)
                        .strong(),
                )
                .corner_radius(egui::CornerRadius::same(4)),
            );
            if btn.clicked() {
                submitted = true;
            }
        });
    });
    submitted
}
