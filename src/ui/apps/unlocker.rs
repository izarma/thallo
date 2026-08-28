use bevy_egui::{
    EguiClipboard,
    egui::{self, TextBuffer},
};

use crate::{
    engine::design_scale::DesignScale,
    ui::theme::{
        palette::{CONTENT_FONT_SIZE, DEEP_RED_THEME, apply_button_theme},
        widgets::primitives::{self},
    },
};

pub(super) fn show_unlocker(
    ui: &mut egui::Ui,
    input: &mut String,
    scale: &DesignScale,
    clipboard: &mut EguiClipboard,
    button_texture: Option<egui::TextureId>,
    reveal_button_texture: Option<egui::TextureId>,
) -> bool {
    let mut submitted = false;
    // Egui state tracking for wrong password feedback.
    // If unlock succeeds, the window app_type changes so this UI never redraws.
    // If we're still here next frame, it failed.
    let failed_id = ui.id().with("unlock_failed");
    let mut failed = ui.data_mut(|d| d.get_temp::<bool>(failed_id).unwrap_or(false));
    let show_plaintext_id = ui.id().with("show_plaintext");
    let mut show_plaintext =
        ui.data_mut(|d| d.get_temp::<bool>(show_plaintext_id).unwrap_or(false));
    ui.vertical_centered(|ui| {
        ui.set_max_width(scale.x * 512.0);
        ui.add_space(scale.py(128.0));
        primitives::header(ui, "This item is Password protected", scale);
        ui.add_space(scale.py(96.0));
        ui.horizontal(|ui| {
            let input_font = egui::FontId::proportional(scale.py(CONTENT_FONT_SIZE));
            ui.add_space(scale.x * 90.0);
            let field = egui::TextEdit::singleline(input)
                .password(!show_plaintext)
                .background_color(DEEP_RED_THEME)
                .hint_text("Enter password")
                .font(input_font)
                .margin(egui::Margin::symmetric(
                    (scale.x * 12.0) as i8,
                    scale.py(12.0) as i8,
                ))
                .desired_width(scale.x * 180.0);
            let response = ui.add(field);
            response.context_menu(|ui| {
                if ui.button("Paste").clicked() {
                    if let Some(text) = clipboard.get_text() {
                        // Match egui's built-in Ctrl+V behavior: replace the current
                        // selection (if any) and insert at the cursor, instead of
                        // overwriting the whole field.
                        let mut state =
                            egui::TextEdit::load_state(ui.ctx(), response.id).unwrap_or_default();
                        let cursor_range = state.cursor.char_range().unwrap_or_else(|| {
                            egui::text::CCursorRange::one(egui::text::CCursor::new(
                                input.chars().count(),
                            ))
                        });

                        let mut ccursor = input.delete_selected(&cursor_range);
                        let single_line = text.replace(['\r', '\n'], " ");
                        input.insert_text_at(&mut ccursor, &single_line, usize::MAX);

                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                        egui::TextEdit::store_state(ui.ctx(), response.id, state);

                        ui.data_mut(|d| d.insert_temp(failed_id, false));
                    }
                    ui.close();
                }
            });
            if response.changed() {
                failed = false;
                ui.data_mut(|d| d.insert_temp(failed_id, false));
            }
            let enter_pressed =
                response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            apply_button_theme(ui);
            // Toggle password visibility.
            let eye_response = primitives::icon_button(
                ui,
                scale,
                egui::vec2(40.0, 40.0),
                reveal_button_texture,
                show_plaintext,
            )
            .on_hover_text("Show/hide password");
            if eye_response.clicked() {
                show_plaintext = !show_plaintext;
            }

            ui.add_space(scale.py(16.0));
            let btn = primitives::button(ui, "Unlock", scale, button_texture);

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
    ui.data_mut(|d| d.insert_temp(show_plaintext_id, show_plaintext));
    submitted
}
