use bevy_egui::egui;

use crate::ui::theme::palette::{BUTTON_TEXT_COLOR, HEADER_COLOR, LABEL_COLOR, apply_button_theme};

/// Wraps `body` in a vertically-and-horizontally centred [`egui::CentralPanel`]
///
/// # Example
/// ```ignore
/// widget::centered_panel(ctx, "shutdown_menu", |ui| {
///     widget::header(ui, "Hello");
///     if widget::button(ui, "OK").clicked() { … }
/// });
/// ```
pub fn centered_panel(ctx: &egui::Context, id: &str, body: impl FnOnce(&mut egui::Ui)) {
    // consume the CentralPanel so egui doesn't complain about unused space
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |_ui| {});

    egui::Area::new(egui::Id::new(id))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 20.0;
            ui.vertical_centered(|ui| {
                body(ui);
            });
        });
}

/// A large header label (≈ 40 px).
pub fn header(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text)
            .size(40.0)
            .color(HEADER_COLOR)
            .strong(),
    );
}

/// A regular text label (≈ 24 px).
pub fn label(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(egui::RichText::new(text).size(24.0).color(LABEL_COLOR));
}

/// A large rounded button (380 × 80).  Returns the [`egui::Response`] so the
/// caller can check `.clicked()`, `.hovered()`, etc.
pub fn button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    apply_button_theme(ui);
    ui.add_sized(
        [380.0, 80.0],
        egui::Button::new(
            egui::RichText::new(text)
                .size(32.0)
                .color(BUTTON_TEXT_COLOR),
        )
        .corner_radius(egui::CornerRadius::same(40)),
    )
}

/// A placeholder label for empty or error states — italicised and dimmed.
pub fn empty_state(ui: &mut egui::Ui, text: &str) {
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(text).italics().weak());
    });
}

/// Truncates a label to `max_chars` codepoints, appending `…` if cut.
/// Uses `chars().count()` so multi-byte characters (emoji, CJK) are counted correctly.
pub fn truncate_label(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        format!("{}…", s.chars().take(max_chars - 1).collect::<String>())
    } else {
        s.to_string()
    }
}
