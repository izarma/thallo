use bevy_egui::egui::{self, Color32, FontId, RichText, ScrollArea};

use crate::engine::{design_scale::DesignScale, file_system::FsPath};

// Texture split: top zone is ~63% (textured dark), bottom ~37% (pure black)
const TOP_ZONE_RATIO: f32 = 0.68;

const COLOR_DIM: Color32 = Color32::from_rgb(120, 120, 120);
const COLOR_OUTPUT: Color32 = Color32::from_rgb(180, 180, 180);
const COLOR_PROMPT: Color32 = Color32::from_rgb(140, 160, 140); // muted green — operator terminal
const COLOR_INPUT: Color32 = Color32::from_rgb(210, 210, 210);
const COLOR_HEADER: Color32 = Color32::from_rgb(100, 110, 100);

const BOOT_LINES: &[&str] = &[
    "ST-OS v4.2.0 [CLASSIFIED BUILD]",
    "Research Division Terminal Access",
    "WARNING: Unauthorized access is a federal offense.",
    "",
    "Authenticating...",
    "Access granted. Welcome, OPERATOR.",
    "Type \'help\' for a list of available commands",
    "",
];

pub(crate) fn show_terminal(
    ui: &mut egui::Ui,
    cwd: &FsPath,
    history: &Vec<String>,
    input: &mut String,
    scale: &DesignScale,
    is_focused: bool,
) -> Option<String> {
    let font = FontId::monospace(scale.py(12.0));
    let font_sm = FontId::monospace(scale.py(10.5));
    let total_rect = ui.available_rect_before_wrap();
    let split_y = total_rect.min.y + total_rect.height() * TOP_ZONE_RATIO;

    let top_rect = egui::Rect::from_min_max(total_rect.min, egui::pos2(total_rect.max.x, split_y));
    let bot_rect = egui::Rect::from_min_max(egui::pos2(total_rect.min.x, split_y), total_rect.max);

    // Top zone: scrollable output
    let mut top_ui = ui.new_child(egui::UiBuilder::new().max_rect(top_rect));
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(
            scale.px(10.0, 0.0).x as i8,
            scale.py(8.0) as i8,
        ))
        .show(&mut top_ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;

                    // Header
                    ui.label(
                        RichText::new("┌─ ST-OS TERMINAL ───────────────────────────────────┐")
                            .font(font_sm.clone())
                            .color(COLOR_HEADER),
                    );
                    ui.add_space(4.0);

                    // Boot lines
                    for line in BOOT_LINES {
                        ui.label(RichText::new(*line).font(font_sm.clone()).color(COLOR_DIM));
                    }

                    // Command history
                    for line in history {
                        let (color, text) = if line.starts_with('>') {
                            // echoed command
                            (COLOR_PROMPT, line.as_str())
                        } else {
                            (COLOR_OUTPUT, line.as_str())
                        };
                        ui.label(RichText::new(text).font(font.clone()).color(color));
                    }
                });
        });

    // Bottom zone: input
    let mut bot_ui = ui.new_child(egui::UiBuilder::new().max_rect(bot_rect));
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(
            scale.px(10.0, 0.0).x as i8,
            scale.py(10.0) as i8,
        ))
        .show(&mut bot_ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;

            // Separator line
            ui.label(
                RichText::new("└────────────────────────────────────────────────────┘")
                    .font(font_sm.clone())
                    .color(COLOR_HEADER),
            );
            ui.add_space(4.0);

            // Prompt + input field
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("OPER@ST-OS:{}/>", cwd.as_str()))
                        .font(font.clone())
                        .color(COLOR_PROMPT),
                );

                let input_id = ui.make_persistent_id("terminal_input");
                let field = egui::TextEdit::singleline(input)
                    .id(input_id)
                    .font(font.clone())
                    .text_color(COLOR_INPUT)
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .cursor_at_end(true);

                let response = ui.add(field);

                // THIS IS ALL FUCKED UP - NEEDS REDO
                if is_focused && !response.has_focus() {
                    response.request_focus();
                }

                if response.has_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !input.trim().is_empty()
                {
                    return Some(input.trim().to_string());
                }
                None
            })
            .inner
        })
        .inner
}
