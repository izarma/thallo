use bevy_egui::egui::{self, FontId, RichText, ScrollArea};

use crate::{
    engine::{design_scale::DesignScale, file_system::FsPath},
    ui::theme::palette::{
        CONTENT_FONT_SIZE, FONT_SMALL, LABEL_COLOR, LABEL_META, RED_CONTRAST_THEME,
    },
};

// Texture split: top zone is ~63% (textured dark), bottom ~37% (pure black)
const TOP_ZONE_RATIO: f32 = 0.68;

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
    command_history: &mut Vec<String>,
    history_index: &mut Option<usize>,
    draft_input: &mut String,
    scale: &DesignScale,
    is_focused: bool,
) -> Option<String> {
    let font = FontId::monospace(scale.py(CONTENT_FONT_SIZE));
    let font_sm = FontId::monospace(scale.py(FONT_SMALL));
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
                            .font(font.clone())
                            .color(RED_CONTRAST_THEME),
                    );
                    ui.add_space(scale.py(4.0));

                    // Boot lines
                    for line in BOOT_LINES {
                        ui.label(RichText::new(*line).font(font_sm.clone()).color(LABEL_META));
                    }

                    // Command history
                    for line in history {
                        let (color, text) = if line.starts_with('>') {
                            // echoed command
                            (RED_CONTRAST_THEME, line.as_str())
                        } else {
                            (LABEL_COLOR, line.as_str())
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
                    .font(font.clone())
                    .color(RED_CONTRAST_THEME),
            );
            ui.add_space(scale.py(4.0));

            // Prompt + input field
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("OPER@ST-OS:{}/>", cwd.as_str()))
                        .font(font.clone())
                        .color(RED_CONTRAST_THEME),
                );

                let input_id = ui.make_persistent_id("terminal_input");
                let field = egui::TextEdit::singleline(input)
                    .id(input_id)
                    .font(font.clone())
                    .text_color(LABEL_COLOR)
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .cursor_at_end(true);

                let response = ui.add(field);

                if is_focused && !response.has_focus() {
                    response.request_focus();
                }

                if response.has_focus() {
                    // Navigate command history like a normal terminal.
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp))
                        && !command_history.is_empty()
                    {
                        if history_index.is_none() {
                            *draft_input = input.clone();
                            *history_index = Some(command_history.len() - 1);
                        } else if let Some(idx) = *history_index {
                            *history_index = Some(idx.saturating_sub(1));
                        }
                        if let Some(idx) = *history_index {
                            *input = command_history[idx].clone();
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown))
                        && let Some(idx) = *history_index
                    {
                        if idx + 1 < command_history.len() {
                            *history_index = Some(idx + 1);
                            *input = command_history[idx + 1].clone();
                        } else {
                            *history_index = None;
                            *input = draft_input.clone();
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && !input.trim().is_empty() {
                        let cmd = input.trim().to_string();
                        command_history.push(cmd.clone());
                        *history_index = None;
                        draft_input.clear();
                        input.clear();
                        return Some(cmd);
                    }
                }
                None
            })
            .inner
        })
        .inner
}
