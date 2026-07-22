use bevy_egui::egui;

use crate::{
    engine::design_scale::DesignScale,
    ui::theme::{
        palette::{CONTENT_FONT_SIZE, DEEP_RED_THEME, RED_CONTRAST_THEME, apply_button_theme},
        widgets::primitives::{self},
    },
};

pub(super) fn show_unlocker(ui: &mut egui::Ui, input: &mut String, scale: &DesignScale) -> bool {
    let mut submitted = false;
    // Egui state tracking for wrong password feedback.
    // If unlock succeeds, the window app_type changes so this UI never redraws.
    // If we're still here next frame, it failed.
    let failed_id = ui.id().with("unlock_failed");
    let mut failed = ui.data_mut(|d| d.get_temp::<bool>(failed_id).unwrap_or(false));
    ui.vertical_centered(|ui| {
        ui.set_max_width(scale.x * 512.0);
        ui.add_space(scale.py(128.0));
        primitives::header(ui, "This item is Password protected", scale);
        ui.add_space(scale.py(96.0));
        ui.horizontal(|ui| {
            let input_font = egui::FontId::proportional(scale.py(CONTENT_FONT_SIZE));
            ui.add_space(scale.x * 90.0);
            let field = egui::TextEdit::singleline(input)
                .password(true)
                .background_color(DEEP_RED_THEME)
                .hint_text("Enter password")
                .font(input_font)
                .margin(egui::Margin::symmetric(
                    (scale.x * 12.0) as i8,
                    scale.py(12.0) as i8,
                ))
                .desired_width(scale.x * 180.0);
            let response = ui.add(field);
            if response.changed() {
                failed = false;
                ui.data_mut(|d| d.insert_temp(failed_id, false));
            }
            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            apply_button_theme(ui);
            ui.add_space(scale.py(16.0));
            let btn = ui.add_sized(
                scale.px(120.0, 48.0),
                egui::Button::new(
                    egui::RichText::new("Unlock")
                        .size(scale.py(CONTENT_FONT_SIZE))
                        .color(RED_CONTRAST_THEME)
                        .strong(),
                ),
            );

            if btn.clicked() || enter_pressed {
                submitted = true;
                // Log failure for next frame
                ui.data_mut(|d| d.insert_temp(failed_id, true));
            }
        });

        // failed feedback
        if failed {
            ui.add_space(scale.x * 16.0);
            ui.label(
                egui::RichText::new("Access Denied: Incorrect Password")
                    .color(egui::Color32::RED)
                    .size(scale.py(14.0)),
            );
        }
    });
    submitted
}
